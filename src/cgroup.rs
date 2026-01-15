use std::fs::{self, OpenOptions};
use std::{error::Error, io::Write};

const CGROUP_CONTROLLER: &str = "/sys/fs/cgroup/cgroup.subtree_control";
const CGROUP_PATH: &str = "/sys/fs/cgroup";

const PID_MAX: &str = "pids.max";
const MEMORY_MAX: &str = "memory.max";
const CPU_MAX: &str = "cpu.max";
const CGROUP_PROCS: &str = "cgroup.procs";

enum Controller {
    Cpu,
    CpuSet,
    Memory,
    Pids,
}

impl Controller {
    fn as_str(&self) -> &str {
        match self {
            Controller::CpuSet => "cpuset",
            Controller::Cpu => "cpu",
            Controller::Memory => "memory",
            Controller::Pids => "pids",
        }
    }
}

/// Builder for creating and configuring cgroups.
///
/// This builder allows setting limits for specific controllers
/// and assigning a process to the created cgroup.
#[derive(Default)]
pub struct CgroupBuilder<'a> {
    name: &'a str,
    memory_limit: Option<&'a [u8]>,
    cpu_limit: Option<&'a [u8]>,
    pids_limit: Option<&'a [u8]>,
    controller: Vec<Controller>,
    pids: Option<u32>,
}

impl<'a> CgroupBuilder<'a> {
    /// Creates a new `CgroupBuilder` with the given cgroup name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the cgroup to create.
    pub fn new(name: &'a str) -> Self {
        CgroupBuilder {
            name: name,
            memory_limit: None,
            cpu_limit: None,
            pids_limit: None,
            controller: vec![],
            pids: None,
        }
    }

    /// Sets the memory limit for the cgroup.
    ///
    /// Also enables the memory controller.
    ///
    /// # Arguments
    ///
    /// * `limit` - The memory limit in bytes.
    pub fn memory_limit(mut self, limit: &'a [u8]) -> Self {
        self.memory_limit = Some(limit);
        self.controller.push(Controller::Memory);
        self
    }

    /// Sets the CPU usage limit for the cgroup.
    ///
    /// Also enables the cpu and cpuset controllers.
    ///
    /// # Arguments
    ///
    /// * `limit` - The cpu limit configuration.
    pub fn cpu_limit(mut self, limit: &'a [u8]) -> Self {
        self.cpu_limit = Some(limit);
        self.controller.push(Controller::Cpu);
        self.controller.push(Controller::CpuSet);
        self
    }

    /// Sets the maximum number of PIDs for the cgroup.
    ///
    /// Also enables the pids controller.
    ///
    /// # Arguments
    ///
    /// * `limit` - The maximum number of PIDs.
    pub fn pids_limit(mut self, limit: &'a [u8]) -> Self {
        self.pids_limit = Some(limit);
        self.controller.push(Controller::Pids);
        self
    }

    /// Sets the given PID to be added to the cgroup.
    ///
    /// # Arguments
    ///
    /// * `pid` - The main process ID of the container.
    pub fn add_task(mut self, pid: u32) -> Self {
        self.pids = Some(pid);
        self
    }

    fn cgroup_controller_configure(&self) -> Result<(), Box<dyn Error>> {
        self.controller.iter().try_for_each(|c| {
            let mut file = OpenOptions::new().write(true).open(CGROUP_CONTROLLER)?;

            file.write_all(format!("+{}", c.as_str()).as_bytes())?;
            Ok(())
        })
    }

    fn cgroup_configure(&self, path: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new()
            .write(true)
            .open(format!("{}/{}/{}", CGROUP_PATH, self.name, path))?;
        file.write_all(data)?;
        Ok(())
    }

    /// Builds and configures the cgroup.
    ///
    /// This method performs the following actions:
    /// 1. Enables necessary controllers in the cgroup root.
    /// 2. Creates the cgroup directory.
    /// 3. Adds the process to the cgroup.
    /// 4. Applies CPU, memory, and PID limits if configured.
    pub fn build(self) -> Result<Self, Box<dyn Error>> {
        self.cgroup_controller_configure()?;

        fs::create_dir_all(format!("{}/{}", CGROUP_PATH, self.name))?;

        if let Some(pid) = self.pids {
            self.cgroup_configure(CGROUP_PROCS, pid.to_string().as_bytes())?;
        }

        if let Some(data) = self.cpu_limit {
            self.cgroup_configure(CPU_MAX, data)?;
        }

        if let Some(data) = self.memory_limit {
            self.cgroup_configure(MEMORY_MAX, data)?;
        }
        if let Some(data) = self.pids_limit {
            self.cgroup_configure(PID_MAX, data)?;
        }

        Ok(CgroupBuilder {
            name: self.name,
            ..Default::default()
        })
    }

    pub fn cleanup(&self) -> Result<(), Box<dyn Error>> {
        let path = format!("{}/{}", CGROUP_PATH, self.name);
        fs::remove_dir(&path)?;
        Ok(())
    }
}
