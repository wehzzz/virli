use nix::mount::{MsFlags, mount};
use std::{error::Error, fs, path::PathBuf};

const VOLUME_DIR_IN_CONTAINER: &str = "home/student";
const ROOT_DIR: &str = "/";

const SYSFS: &str = "sysfs";
const PROCFS: &str = "proc";
const TMPFS: &str = "tmpfs";
const OVERLAYFS: &str = "overlay";

const DEV_PATH: &str = "dev";
const SYS_PATH: &str = "sys";
const PROC_PATH: &str = "proc";
const TMP_PATH: &str = "tmp";

/// Mounts the essential system filesystems (proc, sysfs, tmpfs, dev) inside the container root.
///
/// Use private mount propagation to ensure that mounts inside the container do not affect the host.
///
/// # Arguments
///
/// * `rootfs` - The path to the container's root filesystem.
pub fn mount_sysfs(rootfs: &PathBuf) -> Result<(), Box<dyn Error>> {
    // Make sure mount propagation is private to avoid side effects on the host
    mount(
        None::<&str>,
        ROOT_DIR,
        None::<&str>,
        MsFlags::MS_PRIVATE | MsFlags::MS_REC,
        None::<&str>,
    )?;

    // Mount /proc filesystem
    let proc_path = rootfs.join(PROC_PATH);
    if !proc_path.exists() {
        fs::create_dir_all(&proc_path)?;
    }
    mount(
        Some(PROCFS),
        &proc_path,
        Some(PROCFS),
        MsFlags::empty(),
        None::<&str>,
    )?;

    // Mount /tmp as tmpfs
    let tmp_path = rootfs.join(TMP_PATH);
    if !tmp_path.exists() {
        fs::create_dir_all(&tmp_path)?;
    }
    mount(
        Some(TMPFS),
        &tmp_path,
        Some(TMPFS),
        MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
        None::<&str>,
    )?;

    // Mount /dev as tmpfs
    let dev_path = rootfs.join(DEV_PATH);
    if !dev_path.exists() {
        fs::create_dir_all(&dev_path)?;
    }
    mount(
        Some(TMPFS),
        &dev_path,
        Some(TMPFS),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC,
        None::<&str>,
    )?;

    // Mount /sys filesystem
    let sys_path = rootfs.join(SYS_PATH);
    if !sys_path.exists() {
        fs::create_dir_all(&sys_path)?;
    }
    mount(
        Some(SYSFS),
        &sys_path,
        Some(SYSFS),
        MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_RDONLY,
        None::<&str>,
    )?;

    Ok(())
}

/// Mounts a host directory as a volume inside the container.
///
/// The volume is mounted at `home/student` inside the container root.
///
/// # Arguments
///
/// * `rootfs` - The path to the container's root filesystem.
/// * `path` - Optional path to the directory on the host to mount.
pub fn mount_volume(rootfs: &PathBuf, path: &Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    if !rootfs.exists() {
        return Err("Root filesystem path does not exist".into());
    }

    // Target directory in the container
    let volume = rootfs.join(VOLUME_DIR_IN_CONTAINER);
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

    // Bind mount the volume
    mount(
        Some(volume_to_mount),
        &volume,
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    )?;

    Ok(())
}

/// Sets up an OverlayFS for the container.
///
/// Creates the necessary upper, work, and merged directories within the runtime directory.
/// Mounts the overlay filesystem merging the read-only image layer with the read-write upper layer.
///
/// # Arguments
///
/// * `image_path` - The path to the base image e.g rootfs (lower directory).
/// * `runtime_dir` - The directory where runtime changes and work files will be stored.
///
/// # Returns
///
/// Returns the path to the merged directory which serves as the container's root filesystem.
pub fn mount_overlayfs(
    image_path: &Option<PathBuf>,
    runtime_dir: &PathBuf,
) -> Result<PathBuf, Box<dyn Error>> {
    let image_path = match image_path {
        Some(path) => path,
        None => return Err("Image path not provided".into()),
    };

    // Prepare directories for OverlayFS
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

    // Mount OverlayFS
    mount(
        Some(OVERLAYFS),
        &merged_dir,
        Some(OVERLAYFS),
        MsFlags::empty(),
        Some(options.as_str()),
    )
    .map_err(|e| format!("OverlayFS mount failed: {}", e))?;

    Ok(merged_dir)
}
