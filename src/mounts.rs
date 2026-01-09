use nix::mount::{MsFlags, mount};
use std::error::Error;
use std::fs::{self};
use std::path::PathBuf;

pub fn mount_sysfs(rootfs: &Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    let rootfs_path = match rootfs {
        Some(path) => path,
        None => return Err("Root filesystem not provided".into()),
    };

    mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None::<&str>,
    )?;

    let proc_path = rootfs_path.join("proc");
    if !proc_path.exists() {
        fs::create_dir_all(&proc_path)?;
    }
    mount(
        Some("proc"),
        &proc_path,
        Some("proc"),
        MsFlags::empty(),
        None::<&str>,
    )?;

    let tmp_path = rootfs_path.join("tmp");
    if !tmp_path.exists() {
        fs::create_dir_all(&tmp_path)?;
    }
    mount(
        Some("tmpfs"),
        &tmp_path,
        Some("tmpfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
        None::<&str>,
    )?;

    let dev_path = rootfs_path.join("dev");
    if !dev_path.exists() {
        fs::create_dir_all(&dev_path)?;
    }
    mount(
        Some("tmpfs"),
        &dev_path,
        Some("tmpfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC,
        None::<&str>,
    )?;

    let sys_path = rootfs_path.join("sys");
    if !sys_path.exists() {
        fs::create_dir_all(&sys_path)?;
    }
    mount(
        Some("sysfs"),
        &sys_path,
        Some("sysfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_RDONLY,
        None::<&str>,
    )?;

    Ok(())
}

pub fn mount_volume(
    rootfs: &Option<PathBuf>,
    path: &Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let target_path = match rootfs {
        Some(rfs) => rfs,
        None => return Err("Root filesystem not provided".into()),
    };

    if !target_path.exists() {
        return Err("Root filesystem path does not exist".into());
    }

    let volume = target_path.join("home/student");
    if !volume.exists() {
        fs::create_dir_all(&volume)?;
    }

    let volume_to_mount = match path {
        Some(p) => p,
        None => return Ok(()),
    };

    if !volume_to_mount.exists() {
        return Ok(());
    }

    mount(
        Some(volume_to_mount),
        &volume,
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    )?;

    Ok(())
}
