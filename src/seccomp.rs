use libseccomp::*;
use std::error::Error;

pub struct SeccompBuilder {
    filter: ScmpFilterContext,
}

impl SeccompBuilder {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(SeccompBuilder {
            filter: ScmpFilterContext::new(ScmpAction::Allow)?,
        })
    }

    pub fn add_syscall(mut self, syscall: &str) -> Result<Self, Box<dyn Error>> {
        match ScmpSyscall::from_name(syscall) {
            Ok(sysc) => {
                self.filter.add_rule(ScmpAction::Errno(libc::EPERM), sysc)?;
            }
            Err(_) => {
                eprintln!("Seccomp: Unknown syscall {}", syscall);
            }
        }
        Ok(self)
    }

    pub fn apply(self) -> Result<Self, Box<dyn Error>> {
        self.filter.load()?;

        Ok(self)
    }
}
