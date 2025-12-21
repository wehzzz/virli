pub(crate) mod capabilities;
pub(crate) mod cgroup;
pub(crate) mod chroot;
pub(crate) mod parse;
pub(crate) mod seccomp;

use crate::cgroup::CgroupBuilder;

use nix::unistd::execve;
use std::env;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::process;

fn main() -> Result<(), Box<dyn Error>> {
    let raw_args: Vec<String> = env::args().skip(1).collect();

    let args = match parse::parse_args(&raw_args)? {
        Some(elt) => elt,
        None => return Ok(()), // We want to return in case of -h
    };

    let p_path = CString::new(args.command[0].as_str())?;
    let args_: Vec<CString> = args
        .command
        .iter()
        .map(|s| CString::new(s.as_str()).map_err(|e| Box::new(e) as Box<dyn Error>))
        .collect::<Result<_, _>>()?;
    let p_args: Vec<&CStr> = args_.iter().map(|s| s.as_c_str()).collect();

    let p_env: Vec<&CStr> = vec![];

    let _cgroup = CgroupBuilder::new("mymoulette")
        .memory_limit(b"1073741824")
        .cpu_limit(b"100000 100000")
        .pids_limit(b"100")
        .add_task(process::id())
        .build()?;

    chroot::isolate_fs(args.rootfs)?;

    let _seccomp = seccomp::SeccompBuilder::new()?
        .add_syscall("nfsservctl")?
        .add_syscall("personality")?
        .add_syscall("pivot_root")?
        .apply()?;

    capabilities::capabilities_configure()?;

    execve(p_path.as_c_str(), &p_args, &p_env)?;
    Ok(())
}
