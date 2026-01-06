use nix::sched::{CloneFlags, unshare};
use nix::unistd::{getgid, getuid, sethostname};
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;

pub fn namespace_configure() -> Result<(), Box<dyn Error>> {
    let uid = getuid();
    let gid = getgid();

    unshare(CloneFlags::CLONE_NEWUSER)?;

    fs::write("/proc/self/setgroups", "deny")?;
    fs::write("/proc/self/uid_map", format!("0 {} 1\n", uid))?;
    fs::write("/proc/self/gid_map", format!("0 {} 1\n", gid))?;

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
