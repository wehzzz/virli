pub(crate) mod capabilities;
pub(crate) mod cgroup;
pub(crate) mod chroot;
pub(crate) mod mounts;
pub(crate) mod namespace;
pub(crate) mod oci;
pub(crate) mod parse;
pub(crate) mod seccomp;

use crate::cgroup::CgroupBuilder;

use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{ForkResult, fork};
use std::ffi::{CStr, CString};
use std::{env, error::Error, fs, process};

fn main() -> Result<(), Box<dyn Error>> {
    let raw_args: Vec<String> = env::args().skip(1).collect();

    let args = match parse::parse_args(&raw_args)? {
        Some(elt) => elt,
        None => return Ok(()), // We want to return in case of -h
    };

    let rootfs = match args.image {
        Some(image) => {
            let cache = oci::get_image_path(image);
            match oci::fetch_and_extract_image(&cache, image) {
                Ok(_) => Some(cache),
                Err(e) => {
                    fs::remove_dir_all(&cache)?;
                    return Err(e);
                }
            }
        }
        None => args.rootfs,
    };

    let mut cgroup = CgroupBuilder::new("mymoulette")
        .memory_limit(b"1073741824")
        .cpu_limit(b"100000 100000")
        .pids_limit(b"100")
        .build()
        .map_err(|e| format!("cgroup build: {}", e))?;

    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            cgroup = cgroup
                .add_task(child.as_raw() as u32)
                .build()
                .map_err(|e| format!("cgroup add task: {}", e))?;

            waitpid(child, None).map_err(|e| format!("waitpid supervisor: {}", e))?;

            cgroup
                .cleanup()
                .map_err(|e| format!("cgroup cleanup: {}", e))?;
        }
        ForkResult::Child => {
            namespace::namespace_configure().map_err(|e| format!("namespace configure: {}", e))?;

            match unsafe { fork() }? {
                ForkResult::Parent { child } => match waitpid(child, None)? {
                    WaitStatus::Exited(_, code) => process::exit(code),
                    _ => process::exit(0),
                },
                ForkResult::Child => match child_routine(&args.command, &rootfs, &args.volume) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("Container failed: {}", e);
                        process::exit(1);
                    }
                },
            }
        }
    }

    Ok(())
}

fn child_routine(
    args: &[String],
    rootfs: &Option<std::path::PathBuf>,
    volume: &Option<std::path::PathBuf>,
) -> Result<(), Box<dyn Error>> {
    namespace::setup_hostname().map_err(|e| format!("hostname setup: {}", e))?;

    // We want to create overlayfs in order to make changes in the container without affecting the base image
    let root_overlayfs_path = std::path::PathBuf::from(format!(
        "/tmp/mymoulette_{}",
        namespace::generate_hostname()?
    ));
    std::fs::create_dir_all(&root_overlayfs_path)?;
    let root_overlayfs = mounts::mount_overlayfs(rootfs, &root_overlayfs_path)?;

    mounts::mount_sysfs(&root_overlayfs).map_err(|e| format!("mount_sysfs: {}", e))?;
    mounts::mount_volume(&root_overlayfs, volume).map_err(|e| format!("mount_volume: {}", e))?;

    chroot::isolate_pivot(&root_overlayfs).map_err(|e| format!("pivot_root : {}", e))?;

    let _seccomp = seccomp::SeccompBuilder::new()?
        .add_syscall("nfsservctl")?
        .add_syscall("personality")?
        .add_syscall("pivot_root")?
        .apply()
        .map_err(|e| format!("seccomp: {}", e))?;

    capabilities::capabilities_configure().map_err(|e| format!("capabilities: {}", e))?;

    let args_: Vec<CString> = args
        .iter()
        .map(|s| CString::new(s.as_str()).map_err(|e| Box::new(e) as Box<dyn Error>))
        .collect::<Result<_, _>>()?;
    let p_args: Vec<&CStr> = args_.iter().map(|s| s.as_c_str()).collect();

    nix::unistd::execvp(p_args[0], &p_args)?;

    Ok(())
}
