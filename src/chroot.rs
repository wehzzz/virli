use nix::mount::{MntFlags, MsFlags, mount, umount2};
use nix::unistd::{chdir, chroot, pivot_root};
use std::{error::Error, fs, path::PathBuf};

pub fn _isolate_fs(rootfs: &PathBuf) -> Result<(), Box<dyn Error>> {
    chroot(rootfs)?;
    chdir("/")?;
    Ok(())
}

// https://man7.org/linux/man-pages/man2/pivot_root.2.html
pub fn isolate_pivot(new_root: &PathBuf) -> Result<(), Box<dyn Error>> {
    mount(
        None::<&str>,
        "/",
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

    chdir("/").map_err(|e| format!("Chdir failed: {}", e))?;

    umount2("/put_old", MntFlags::MNT_DETACH)
        .map_err(|e| format!("Umount old root failed: {}", e))?;

    fs::remove_dir("/put_old")?;
    Ok(())
}
