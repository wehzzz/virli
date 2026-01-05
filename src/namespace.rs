use nix::mount::{MsFlags, mount};
use nix::sched::{CloneFlags, unshare};
use nix::unistd::{getgid, getuid};
use std::error::Error;
use std::fs::{self};
use std::path::PathBuf;

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

pub fn setup_mounts(rootfs: &Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    let rootfs = match rootfs {
        Some(path) => path,
        None => return Err("Root filesystem not provided".into()),
    };

    mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None::<&str>,
    )
    .map_err(|e| format!("mount MS_PRIVATE /: {}", e))?;

    let proc_path = rootfs.join("proc");
    fs::create_dir_all(&proc_path)?;
    mount(
        Some("proc"),
        proc_path.as_path(),
        Some("proc"),
        MsFlags::empty(),
        None::<&str>,
    )
    .map_err(|e| format!("mount proc: {}", e))?;

    let sys_path = rootfs.join("sys");
    fs::create_dir_all(&sys_path)?;
    mount(
        Some("sysfs"),
        sys_path.as_path(),
        Some("sysfs"),
        MsFlags::empty(),
        None::<&str>,
    )
    .map_err(|e| format!("mount sysfs: {}", e))?;

    let dev_path = rootfs.join("dev");
    fs::create_dir_all(&dev_path)?;
    mount(
        Some("tmpfs"),
        dev_path.as_path(),
        Some("tmpfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC,
        Some("mode=755"),
    )
    .map_err(|e| format!("mount tmpfs /dev: {}", e))?;

    Ok(())
}
