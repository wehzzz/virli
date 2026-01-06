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
use std::env;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fs;
use std::process;

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

    let _cgroup = CgroupBuilder::new("mymoulette")
        .memory_limit(b"1073741824")
        .cpu_limit(b"100000 100000")
        .pids_limit(b"100")
        .add_task(process::id())
        .build()
        .ok();

    namespace::namespace_configure()?;

    match unsafe { fork() }? {
        ForkResult::Parent { child } => match waitpid(child, None)? {
            WaitStatus::Exited(_, code) => {
                if code != 0 {
                    println!("Container exited with code {}", code);
                }
            }
            _ => {}
        },
        ForkResult::Child => match child_routine(&args.command, &rootfs, &args.volume) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Container failed: {}", e);
                process::exit(1);
            }
        },
    }

    Ok(())
}

fn child_routine(
    args: &[String],
    rootfs: &Option<std::path::PathBuf>,
    volume: &Option<std::path::PathBuf>,
) -> Result<(), Box<dyn Error>> {
    namespace::setup_hostname()?;

    mounts::mount_sysfs(rootfs)?;
    mounts::mount_volume(rootfs, volume)?;

    chroot::isolate_fs(rootfs)?;

    let _seccomp = seccomp::SeccompBuilder::new()?
        .add_syscall("nfsservctl")?
        .add_syscall("personality")?
        .add_syscall("pivot_root")?
        .apply()?;

    capabilities::capabilities_configure()?;

    let args_: Vec<CString> = args
        .iter()
        .map(|s| CString::new(s.as_str()).map_err(|e| Box::new(e) as Box<dyn Error>))
        .collect::<Result<_, _>>()?;
    let p_args: Vec<&CStr> = args_.iter().map(|s| s.as_c_str()).collect();

    nix::unistd::execvp(p_args[0], &p_args)?;

    Ok(())
}
