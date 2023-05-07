pub mod algo;
pub mod sim;
pub mod structures;
pub mod structures2;

use sim::{OrderKind, SimProcess};
use std::io::{self, BufRead, BufReader};
use std::num::ParseIntError;
use std::{fs, result};

#[derive(Debug)]
pub enum ProgramError {
    IOError(io::Error),
    InvalidProcessSpecification(String),
    InvalidProcessParseError(ParseIntError),
    InvalidCommandInput,
    GeneralError,
}
pub type Result<T> = result::Result<T, ProgramError>;

impl From<io::Error> for ProgramError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<ParseIntError> for ProgramError {
    fn from(value: ParseIntError) -> Self {
        Self::InvalidProcessParseError(value)
    }
}

#[derive(Debug)]
pub enum ScheduleKind {
    FCFS,
    SJF,
    Priority,
    RR,
    PriorityRR,
}

pub struct Configuration {
    pub scheduler: ScheduleKind,
    pub filename: String,
}

impl Configuration {
    pub fn build(args: &Vec<String>) -> Result<Self> {
        let mut iter = args.into_iter();
        iter.next();

        let in_filename = match iter.next() {
            Some(str) => String::from(str),
            None => {
                eprintln!("<process-filename> is a required argument");
                return Err(ProgramError::InvalidCommandInput);
            }
        };

        let scheduler = match iter.next() {
            Some(number) => match number.parse::<u8>()? {
                0 => ScheduleKind::FCFS,
                1 => ScheduleKind::SJF,
                2 => ScheduleKind::Priority,
                3 => ScheduleKind::RR,
                4 => ScheduleKind::PriorityRR,
                _ => return Err(ProgramError::InvalidCommandInput),
            },
            None => ScheduleKind::FCFS,
        };

        Ok(Self {
            scheduler,
            filename: in_filename,
        })
    }
}

pub fn print_usage_statement(args: Vec<String>) {
    println!("usage: {} <process-filename> <scheduler-type-id>", args[0]);
    println!("received: {:?}", args);
}

pub fn read_processes(ordering: OrderKind, filename: &str) -> Result<Vec<SimProcess>> {
    let file = fs::File::open(filename)?;
    let reader = BufReader::new(file);
    let order_key = match ordering {
        OrderKind::Burst => 0,
        OrderKind::Priority => 1,
    };
    reader
        .lines()
        .map(|line| SimProcess::try_from(format!("{},{}", line?, order_key)))
        .collect::<Result<Vec<SimProcess>>>()
}

pub fn display_processes(processes: &Vec<SimProcess>) {
    for process in processes {
        println!("{}", process);
    }
}

#[allow(dead_code)]
const DEFAULT_PROCESS_FILENAME: &str = "process-list.txt";

#[cfg(test)]
mod tests {
    use super::*;

    mod io_tests {

        use super::*;

        #[test]
        fn read_processes_ioerror_invalid_file() {
            let filename = "does-not-exist.txt";
            let unnecessary_ordering = OrderKind::Burst;
            let result = read_processes(unnecessary_ordering, filename);
            match result {
                Err(ProgramError::IOError(_)) => (),
                val => panic!("received unexpected value: {:?}", val),
            };
        }

        #[test]
        fn read_processes_valid_file() {
            let filename = DEFAULT_PROCESS_FILENAME;
            let unnecessary_ordering = OrderKind::Burst;

            let result = read_processes(unnecessary_ordering, filename);
            match result {
                Ok(_) => (),
                val => panic!("received unexpected value: {:?}", val),
            };
        }
    }

    mod config_tests {}
}
