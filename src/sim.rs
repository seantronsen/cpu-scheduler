use crate::{ProgramError, Result};
use std::io;

pub struct SimProcess {
    pub name: String,
    pub priority: u8,
    pub burst: u32,
    pub wait: u32,
}

impl std::fmt::Display for SimProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Process: {} | Priority: {} | Remaining Burst: {} | Wait Time: {}",
            self.name, self.priority, self.burst, self.wait,
        )
    }
}

impl TryFrom<io::Result<String>> for SimProcess {
    type Error = ProgramError;
    fn try_from(value: io::Result<String>) -> Result<Self> {
        let value = value?;
        let mut components = value.split(",").map(|s| s.trim().to_string());
        let name = match components.next() {
            Some(str) => str,
            _ => return Err(ProgramError::InvalidProcessSpecification(value.to_string())),
        };

        let priority = match components.next() {
            Some(str) => str.parse::<u8>()?,
            _ => return Err(ProgramError::InvalidProcessSpecification(value.to_string())),
        };

        let burst = match components.next() {
            Some(str) => str.parse::<u32>()?,
            _ => return Err(ProgramError::InvalidProcessSpecification(value.to_string())),
        };

        Ok(Self {
            name,
            priority,
            burst,
            wait: 0,
        })
    }
}
