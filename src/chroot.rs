use nix::mount::{MntFlags, MsFlags, mount, umount2};
use nix::unistd::{chdir, chroot, pivot_root};
use std::{error::Error, fs, path::PathBuf};

const PUT_OLD_DIR: &str = "/put_old";
const ROOT_DIR: &str = "/";

/// Old isolation function.
/// Changes the root directory of the calling process to the specified path.
///
/// This is a wrapper around the `chroot` system call.
pub fn _isolate_fs(rootfs: &PathBuf) -> Result<(), Box<dyn Error>> {
    chroot(rootfs)?;
    chdir("/")?;
    Ok(())
}

/// https://man7.org/linux/man-pages/man2/pivot_root.2.html
/// Isolates the filesystem using `pivot_root`.
///
/// # Arguments
///
/// * `new_root` - The new root filesystem path.
pub fn isolate_pivot(new_root: &PathBuf) -> Result<(), Box<dyn Error>> {
    mount(
        None::<&str>,
        ROOT_DIR,
        None::<&str>,
        MsFlags::MS_REC | MsFlags::MS_PRIVATE,
        None::<&str>,
    )
    .map_err(|e| format!("Cannot make mount private: {}", e))?;

    mount(
        Some(new_root),
        new_root,
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    )
    .map_err(|e| format!("Cannot bind mount new root: {}", e))?;

    let put_old = new_root.join("put_old");
    if !put_old.exists() {
        fs::create_dir(&put_old)?;
    }

    pivot_root(new_root, &put_old).map_err(|e| format!("Pivot root failed: {}", e))?;

    chdir(ROOT_DIR).map_err(|e| format!("Chdir failed: {}", e))?;

    umount2(PUT_OLD_DIR, MntFlags::MNT_DETACH)
        .map_err(|e| format!("Umount old root failed: {}", e))?;

    fs::remove_dir(PUT_OLD_DIR)?;
    Ok(())
}
