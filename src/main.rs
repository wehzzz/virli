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

const TMP_DIR_PREFIX: &str = "/tmp/mymoulette_";

const CGROUP_NAME: &str = "mymoulette";
const MEMORY_LIMIT: &str = "1073741824";
const CPU_LIMIT: &str = "100000 100000";
const PIDS_LIMIT: &str = "100";

fn main() -> Result<(), Box<dyn Error>> {
    let raw_args: Vec<String> = env::args().skip(1).collect();

    // Parse command line arguments
    let args = match parse::parse_args(&raw_args)? {
        Some(elt) => elt,
        None => return Ok(()), // We want to return in case of -h
    };

    // Prepare rootfs, either from a provided rootfs or by fetching a Docker image
    let rootfs = match args.image {
        Some(image) => {
            let cache = oci::get_image_path(image);
            match oci::fetch_and_extract_image(&cache, image) {
                Ok(_) => Some(cache),
                Err(e) => {
                    if let Err(cleanup_err) = fs::remove_dir_all(&cache) {
                        eprintln!(
                            "Failed to clean up cache directory {:?} after image error: {}. Cleanup error: {}",
                            &cache, e, cleanup_err
                        );
                    }
                    return Err(e);
                }
            }
        }
        None => args.rootfs,
    };

    let mut cgroup = CgroupBuilder::new(CGROUP_NAME)
        .memory_limit(MEMORY_LIMIT.as_bytes())
        .cpu_limit(CPU_LIMIT.as_bytes())
        .pids_limit(PIDS_LIMIT.as_bytes())
        .build()
        .map_err(|e| format!("cgroup build: {}", e))?;

    // First Fork: Create a supervisor process in order to manage the container lifecycle
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

            // Second Fork: Necessary to properly become PID 1 in the new PID namespace
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
    // This allows the container to have a read-write layer over the read-only image
    let root_overlayfs_path = std::path::PathBuf::from(format!(
        "{}{}",
        TMP_DIR_PREFIX,
        namespace::generate_hostname()?
    ));
    std::fs::create_dir_all(&root_overlayfs_path)?;
    let root_overlayfs = mounts::mount_overlayfs(rootfs, &root_overlayfs_path)?;

    mounts::mount_sysfs(&root_overlayfs).map_err(|e| format!("mount_sysfs: {}", e))?;
    mounts::mount_volume(&root_overlayfs, volume).map_err(|e| format!("mount_volume: {}", e))?;

    chroot::isolate_pivot(&root_overlayfs).map_err(|e| format!("pivot_root : {}", e))?;

    let _seccomp = seccomp::SeccompBuilder::new()?
        .add_syscall(seccomp::Syscall::Nfsservctl)?
        .add_syscall(seccomp::Syscall::Personality)?
        .add_syscall(seccomp::Syscall::PivotRoot)?
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
