use libc::{PR_SET_DUMPABLE, prctl};
use nix::sched::{CloneFlags, unshare};
use nix::unistd::{getgid, getuid, sethostname};
use std::{
    error::Error,
    fs::{self, File},
    io::Read,
};

const HOSTNAME_LENGTH: usize = 12;
const HOSTNAME_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const HOSTNAME_FALLBACK: &str = "mymoulette";
const RANDOM_DEVICE: &str = "/dev/urandom";

const SETGROUPS_PATH: &str = "/proc/self/setgroups";
const UID_MAP_PATH: &str = "/proc/self/uid_map";
const GID_MAP_PATH: &str = "/proc/self/gid_map";

/// Configures Linux namespaces internally within the process.
///
/// This moves the process into new namespaces:
/// * User
/// * Mount
/// * PID
/// * IPC
/// * Network
/// * UTS
/// * Cgroup
///
/// It also handles the user mapping setup required for unprivileged user namespaces.
pub fn namespace_configure() -> Result<(), Box<dyn Error>> {
    let uid = getuid();
    let gid = getgid();

    unshare(CloneFlags::CLONE_NEWUSER)?;
    // Adding capabilities with sudo to our binary set the dumpable flag to 0 by default
    // We need to set it back to 1 to be able to write uid_map and gid_map
    let prctl_ret = unsafe { prctl(PR_SET_DUMPABLE, 1, 0, 0, 0) };
    if prctl_ret != 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    fs::write(SETGROUPS_PATH, "deny").map_err(|e| format!("write setgroups: {}", e))?;
    fs::write(UID_MAP_PATH, format!("0 {} 1\n", uid))
        .map_err(|e| format!("write uid_map: {}", e))?;
    fs::write(GID_MAP_PATH, format!("0 {} 1\n", gid))
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

/// Generates a random hostname string.
///
/// Returns a 12-character alphanumeric string.
pub fn generate_hostname() -> Result<String, Box<dyn Error>> {
    let mut file = File::open(RANDOM_DEVICE)?;
    let mut buffer = [0u8; HOSTNAME_LENGTH];
    file.read_exact(&mut buffer)?;

    let chars: &[u8] = HOSTNAME_CHARS;

    let hostname: String = buffer
        .iter()
        .map(|&byte| {
            let idx = (byte as usize) % chars.len();
            chars[idx] as char
        })
        .collect();

    Ok(hostname)
}

/// Sets the container's hostname.
///
/// Generates a random hostname and applies it using `sethostname`.
pub fn setup_hostname() -> Result<(), Box<dyn Error>> {
    let hostname = match generate_hostname() {
        Ok(name) => name,
        Err(_) => HOSTNAME_FALLBACK.to_string(),
    };

    sethostname(hostname)?;
    Ok(())
}
