use nix::unistd::{chdir, chroot};
use std::error::Error;
use std::path::PathBuf;

pub fn isolate_fs(rootfs: &Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    if let Some(path) = rootfs {
        chroot(path)?;
        chdir("/")?;
    }
    Ok(())
}
