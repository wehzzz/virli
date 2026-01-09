use libc::{PR_SET_DUMPABLE, prctl};
use nix::sched::{CloneFlags, unshare};
use nix::unistd::{getgid, getuid, sethostname};
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;

pub fn namespace_configure() -> Result<(), Box<dyn Error>> {
    let uid = getuid();
    let gid = getgid();

    unshare(CloneFlags::CLONE_NEWUSER)?;
    // Adding capabilities with sudo to our binary set the dumpable flag to 0 by default
    // We need to set it back to 1 to be able to write uid_map and gid_map
    unsafe {
        prctl(PR_SET_DUMPABLE, 1, 0, 0, 0);
    }

    let _ = fs::write("/proc/self/setgroups", "deny");
    fs::write("/proc/self/uid_map", format!("0 {} 1\n", uid))
        .map_err(|e| format!("write uid_map: {}", e))?;
    fs::write("/proc/self/gid_map", format!("0 {} 1\n", gid))
        .map_err(|e| format!("write gid_map: {}", e))?;

    let flags = CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWPID
        | CloneFlags::CLONE_NEWIPC
        | CloneFlags::CLONE_NEWNET
        | CloneFlags::CLONE_NEWUTS
        | CloneFlags::CLONE_NEWCGROUP;

    unshare(flags)?;

    Ok(())
}

fn generate_hostname() -> Result<String, Box<dyn Error>> {
    let mut file = File::open("/dev/urandom")?;
    let mut buffer = [0u8; 12];
    file.read_exact(&mut buffer)?;

    let chars: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    let hostname: String = buffer
        .iter()
        .map(|&byte| {
            let idx = (byte as usize) % chars.len();
            chars[idx] as char
        })
        .collect();

    Ok(hostname)
}

pub fn setup_hostname() -> Result<(), Box<dyn Error>> {
    let hostname = match generate_hostname() {
        Ok(name) => name,
        Err(_) => "moulette-fallback".to_string(),
    };

    sethostname(hostname)?;
    Ok(())
}
