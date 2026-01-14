use libseccomp::*;
use std::error::Error;

/// Builder for configuring Seccomp syscall filtering.
pub struct SeccompBuilder {
    filter: ScmpFilterContext,
}

pub enum Syscall {
    Nfsservctl,
    Personality,
    PivotRoot,
}

impl Syscall {
    fn as_str(&self) -> &str {
        match self {
            Syscall::Nfsservctl => "nfsservctl",
            Syscall::Personality => "personality",
            Syscall::PivotRoot => "pivot_root",
        }
    }
}

impl SeccompBuilder {
    /// Creates a new `SeccompBuilder`.
    ///
    /// The default action is set to `Allow`, meaning all syscalls are permitted
    /// unless explicitly blocked.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(SeccompBuilder {
            filter: ScmpFilterContext::new(ScmpAction::Allow)?,
        })
    }

    /// Adds a rule to block a specific syscall.
    ///
    /// If the syscall name is valid, it adds a rule to return `EPERM`
    /// when the syscall is invoked.
    ///
    /// # Arguments
    ///
    /// * `syscall` - The name of the syscall to block.
    pub fn add_syscall(mut self, syscall: Syscall) -> Result<Self, Box<dyn Error>> {
        match ScmpSyscall::from_name(syscall.as_str()) {
            Ok(sysc) => {
                self.filter.add_rule(ScmpAction::Errno(libc::EPERM), sysc)?;
            }
            Err(_) => {
                eprintln!("Seccomp: Unknown syscall {}", syscall.as_str());
            }
        }
        Ok(self)
    }

    /// Loads the seccomp filter into the kernel.
    ///
    /// Once applied, the filter is active for the current process and its children.
    pub fn apply(self) -> Result<Self, Box<dyn Error>> {
        self.filter.load()?;

        Ok(self)
    }
}
