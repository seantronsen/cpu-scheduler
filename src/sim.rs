use crate::{ProgramError, Result};

#[derive(Debug)]
pub enum OrderKind {
    Burst,
    Priority,
}

impl std::fmt::Display for OrderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OrderKind::Burst => "Burst",
                OrderKind::Priority => "Priority",
            }
        )
    }
}

#[derive(Debug)]
pub struct SimProcess {
    pub name: String,
    pub priority: u8,
    pub remaining_burst: u32,
    running_time: u32,
    pub wait: u32,
    order: OrderKind,
}

impl std::fmt::Display for SimProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Process: {} | Priority: {} | Running Time: {} | Remaining Burst: {} | Wait Time: {} | Order: {}",
            self.name, self.priority, self.running_time, self.remaining_burst, self.wait, self.order,
        )
    }
}

impl TryFrom<String> for SimProcess {
    type Error = ProgramError;
    fn try_from(value: String) -> Result<Self> {
        let mut components = value.split(",").map(|s| String::from(s.trim()));
        let name = match components.next() {
            Some(str) => str,
            _ => {
                return Err(ProgramError::InvalidProcessSpecification(String::from(
                    value,
                )))
            }
        };

        let priority = match components.next() {
            Some(str) => str.parse::<u8>()?,
            _ => {
                return Err(ProgramError::InvalidProcessSpecification(String::from(
                    value,
                )))
            }
        };

        let burst = match components.next() {
            Some(str) => str.parse::<u32>()?,
            _ => {
                return Err(ProgramError::InvalidProcessSpecification(String::from(
                    value,
                )))
            }
        };
        let order = match components.next() {
            Some(str) => match str.parse::<u8>()? {
                0 => OrderKind::Burst,
                1 => OrderKind::Priority,
                _ => {
                    return Err(ProgramError::InvalidProcessSpecification(String::from(
                        value,
                    )))
                }
            },
            _ => {
                return Err(ProgramError::InvalidProcessSpecification(String::from(
                    value,
                )))
            }
        };

        Ok(SimProcess::new(name, priority, burst, order))
    }
}

impl PartialOrd for SimProcess {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.order {
            OrderKind::Burst => self.remaining_burst.partial_cmp(&other.remaining_burst),
            OrderKind::Priority => self.priority.partial_cmp(&other.priority),
        }
    }
}

impl PartialEq for SimProcess {
    fn eq(&self, other: &Self) -> bool {
        match self.order {
            OrderKind::Burst => self.remaining_burst == other.remaining_burst,
            OrderKind::Priority => self.priority == other.priority,
        }
    }
}

impl SimProcess {
    fn new(name: String, priority: u8, burst: u32, order: OrderKind) -> Self {
        Self {
            name,
            priority,
            remaining_burst: burst,
            wait: 0,
            running_time: 0,
            order,
        }
    }

    /// wait time is measured before a process is run, not afterward
    /// w(r, t) = t - r
    ///
    /// where:
    /// - r is the total previous runtime
    /// - t is the current time (i.e. time when the process switches to the running state)
    pub fn run_burst(&mut self, time_at_start: u32, burst: u32) {
        println!("ENTERING: {}", &self);
        let wait_time = time_at_start - self.running_time;
        self.wait = wait_time;
        self.running_time += burst;
        self.remaining_burst -= burst;
        println!(
            "Time: {} | Burst Complete for {}",
            time_at_start + burst,
            &self
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_reference_process() -> SimProcess {
        SimProcess::new(String::from("T1"), 5, 25, OrderKind::Burst)
    }

    #[test]
    fn parse_valid_process_string() -> Result<()> {
        let line = "T1,5,25,1".to_string();
        let process = SimProcess::try_from(line);

        assert_eq!(build_reference_process(), process?);

        Ok(())
    }

    #[test]
    fn parse_error_for_invalid_process_string() {
        let line = String::from("T1, 23, ");
        assert!(SimProcess::try_from(line).is_err());

        let line = String::from("T1, 5, abc");
        assert!(SimProcess::try_from(line).is_err());

        let line = String::from("T1, 5, 25, 8");
        assert!(SimProcess::try_from(line).is_err());
    }

    #[test]
    fn valid_display() {
        let reference_display_string =
            "Process: T1 | Priority: 5 | Running Time: 0 | Remaining Burst: 25 | Wait Time: 0 | Order: Burst";
        assert_eq!(
            build_reference_process().to_string(),
            reference_display_string
        );
    }
}
