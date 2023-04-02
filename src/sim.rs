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
    pub burst: u32,
    pub wait: u32,
    order: OrderKind,
}

impl std::fmt::Display for SimProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Process: {} | Priority: {} | Remaining Burst: {} | Wait Time: {} | Order: {}",
            self.name, self.priority, self.burst, self.wait, self.order,
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
            OrderKind::Burst => self.burst.partial_cmp(&other.burst),
            OrderKind::Priority => self.priority.partial_cmp(&other.priority),
        }
    }
}

impl PartialEq for SimProcess {
    fn eq(&self, other: &Self) -> bool {
        match self.order {
            OrderKind::Burst => self.burst == other.burst,
            OrderKind::Priority => self.priority == other.priority,
        }
    }
}

impl SimProcess {
    fn new(name: String, priority: u8, burst: u32, order: OrderKind) -> Self {
        Self {
            name,
            priority,
            burst,
            wait: 0,
            order,
        }
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
            "Process: T1 | Priority: 5 | Remaining Burst: 25 | Wait Time: 0 | Order: Burst";
        assert_eq!(
            build_reference_process().to_string(),
            reference_display_string
        );
    }
}
