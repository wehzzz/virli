use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;

const CGROUP_CONTROLLER: &str = "/sys/fs/cgroup/cgroup.subtree_control";
const CGROUP_PATH: &str = "/sys/fs/cgroup";

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

pub struct CgroupBuilder<'a> {
    name: &'a str,
    memory_limit: Option<&'a [u8]>,
    cpu_limit: Option<&'a [u8]>,
    pids_limit: Option<&'a [u8]>,
    controller: Vec<Controller>,
    pids: Option<u32>,
}

impl<'a> CgroupBuilder<'a> {
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

    pub fn memory_limit(mut self, limit: &'a [u8]) -> Self {
        self.memory_limit = Some(limit);
        self.controller.push(Controller::Memory);
        self
    }

    pub fn cpu_limit(mut self, limit: &'a [u8]) -> Self {
        self.cpu_limit = Some(limit);
        self.controller.push(Controller::Cpu);
        self.controller.push(Controller::CpuSet);
        self
    }

    pub fn pids_limit(mut self, limit: &'a [u8]) -> Self {
        self.pids_limit = Some(limit);
        self.controller.push(Controller::Pids);
        self
    }

    pub fn add_task(mut self, pid: u32) -> Self {
        self.pids = Some(pid);
        self
    }

    fn cgroup_controller_configure(&self) -> Result<(), Box<dyn Error>> {
        self.controller.iter().try_for_each(|c| {
            let mut file = OpenOptions::new().write(true).open(CGROUP_CONTROLLER)?;

            file.write_all(format!("+{}", c.as_str()).as_bytes())?;
            Ok::<(), Box<dyn Error>>(())
        })
    }

    fn cgroup_configure(&self, path: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let mut file = OpenOptions::new()
            .write(true)
            .open(format!("{}/{}/{}", CGROUP_PATH, self.name, path))?;
        file.write_all(data)?;
        Ok(())
    }

    pub fn build(self) -> Result<Self, Box<dyn Error>> {
        self.cgroup_controller_configure()?;

        fs::create_dir_all(format!("{}/{}", CGROUP_PATH, self.name))?;

        if let Some(pid) = self.pids {
            self.cgroup_configure("cgroup.procs", pid.to_string().as_bytes())?;
        }

        if let Some(data) = self.cpu_limit {
            self.cgroup_configure("cpu.max", data)?;
        }

        if let Some(data) = self.memory_limit {
            self.cgroup_configure("memory.max", data)?;
        }
        if let Some(data) = self.pids_limit {
            self.cgroup_configure("pids.max", data)?;
        }

        Ok(CgroupBuilder {
            name: self.name,
            memory_limit: None,
            cpu_limit: None,
            pids_limit: None,
            controller: vec![],
            pids: None,
        })
    }

    pub fn cleanup(&self) -> Result<(), Box<dyn Error>> {
        let path = format!("{}/{}", CGROUP_PATH, self.name);
        fs::remove_dir(&path)?;
        Ok(())
    }
}
