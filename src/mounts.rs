use nix::mount::{MsFlags, mount};
use std::{error::Error, fs, path::PathBuf};

pub fn mount_sysfs(rootfs: &PathBuf) -> Result<(), Box<dyn Error>> {
    mount(
        None::<&str>,
        "/",
        None::<&str>,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None::<&str>,
    )?;

    let proc_path = rootfs.join("proc");
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

    let tmp_path = rootfs.join("tmp");
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

    let dev_path = rootfs.join("dev");
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

    let sys_path = rootfs.join("sys");
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

pub fn mount_volume(rootfs: &PathBuf, path: &Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    if !rootfs.exists() {
        return Err("Root filesystem path does not exist".into());
    }

    let volume = rootfs.join("home/student");
    if !volume.exists() {
        fs::create_dir_all(&volume)?;
    }

    let volume_to_mount = match path {
        Some(p) => p,
        None => return Ok(()),
    };

    if !volume_to_mount.exists() {
        return Err("Volume path does not exist".into());
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

pub fn mount_overlayfs(
    image_path: &Option<PathBuf>,
    runtime_dir: &PathBuf,
) -> Result<PathBuf, Box<dyn Error>> {
    let image_path = match image_path {
        Some(path) => path,
        None => return Err("Image path not provided".into()),
    };
    let lower_dir = image_path;
    let upper_dir = runtime_dir.join("upper");
    let work_dir = runtime_dir.join("work");
    let merged_dir = runtime_dir.join("merged");

    fs::create_dir_all(&upper_dir)?;
    fs::create_dir_all(&work_dir)?;
    fs::create_dir_all(&merged_dir)?;

    let options = format!(
        "lowerdir={},upperdir={},workdir={}",
        lower_dir.to_str().unwrap(),
        upper_dir.to_str().unwrap(),
        work_dir.to_str().unwrap()
    );

    mount(
        Some("overlay"),
        &merged_dir,
        Some("overlay"),
        MsFlags::empty(),
        Some(options.as_str()),
    )
    .map_err(|e| format!("OverlayFS mount failed: {}", e))?;

    Ok(merged_dir)
}
