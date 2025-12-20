use nix::unistd::{chdir, chroot};
use std::error::Error;

pub fn isolate_fs(rootfs: Option<&str>) -> Result<(), Box<dyn Error>> {
    if let Some(path) = rootfs {
        chroot(path)?;
        chdir("/")?;
    }
    Ok(())
}
