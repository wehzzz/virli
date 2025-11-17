pub(crate) mod cgroup;

use crate::cgroup::CgroupBuilder;

use nix::unistd::execve;
use std::env;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::process;

const USAGE: &str = "MyMoulette, the students'nightmare, now highly secured
 Usage: ./mymoulette [-v student_workdir] <-I docker-img|rootfs-path>
 moulette_prog [moulette_arg [...] ]
    rootfs-path is the path to the directory containing the new rootfs (exclusive
 with -I option)
    docker-img is an image available on hub.docker.com (exclusive with rootfs path)
 moulette_prog will be the first program to be launched, must already be in
 the environment
    student_workdir is the directory containing the code to grade";

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 1 {
        eprintln!("{}", USAGE);
        return Err(("Too few arguments").into());
    }

    if args.get(0) == Some(&"-h".to_string()) {
        eprintln!("{}", USAGE);
        return Ok(());
    }

    let p_path = CString::new(args[0].as_str())?;
    let args_: Vec<CString> = args
        .iter()
        .map(|s| CString::new(s.as_str()).map_err(|e| Box::new(e) as Box<dyn Error>))
        .collect::<Result<_, _>>()?;
    let p_args: Vec<&CStr> = args_.iter().map(|s| s.as_c_str()).collect();

    let p_env: Vec<&CStr> = vec![];

    let cgroup = CgroupBuilder::new("mymoulette")
        .memory_limit(b"1G")?
        .cpu_limit(b"1000000 1000000")?
        .pids_limit(b"100")?
        .build()?;

    cgroup.add_task(process::id())?;

    execve(p_path.as_c_str(), &p_args, &p_env)?;
    Ok(())
}
