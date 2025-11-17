use nix::unistd::execve;
use std::env;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fs::{self, File, OpenOptions};
use std::io::Write;

const USAGE: &str = "MyMoulette, the students'nightmare, now highly secured
 Usage: ./mymoulette [-v student_workdir] <-I docker-img|rootfs-path>
 moulette_prog [moulette_arg [...] ]
    rootfs-path is the path to the directory containing the new rootfs (exclusive
 with -I option)
    docker-img is an image available on hub.docker.com (exclusive with rootfs path)
 moulette_prog will be the first program to be launched, must already be in
 the environment
    student_workdir is the directory containing the code to grade";

const CGROUP_NAME: &str = "/sys/fs/cgroup/mymoulette";

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
    let _args: Vec<CString> = args
        .iter()
        .map(|s| CString::new(s.as_str()).map_err(|e| Box::new(e) as Box<dyn Error>))
        .collect::<Result<_, _>>()?;
    let p_args: Vec<&CStr> = _args.iter().map(|s| s.as_c_str()).collect();

    let p_env: Vec<&CStr> = vec![];

    cgroup()?;

    execve(p_path.as_c_str(), &p_args, &p_env)?;
    Ok(())
}

fn cgroup() -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(CGROUP_NAME)?;

    cgroup_configure("memory.max", b"1G")?;
    cgroup_configure("cpuset.cpus", b"1")?;
    cgroup_configure("pids.max", b"100")?;
    Ok(())
}

fn cgroup_configure(path: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new()
        .write(true)
        .open(format!("{}/{}", CGROUP_NAME, path))?;
    file.write_all(data)?;
    Ok(())
}
