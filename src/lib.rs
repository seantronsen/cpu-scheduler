pub mod sim;

use sim::SimProcess;
use std::io::{self, BufRead, BufReader};
use std::num::ParseIntError;
use std::{fs, result};

const PROCESS_FILENAME: &str = "process-list.txt";
#[derive(Debug)]
pub enum ProgramError {
    IOError(io::Error),
    InvalidProcessSpecification(String),
    InvalidProcessParseError(ParseIntError),
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

pub fn read_processes() -> Result<Vec<SimProcess>> {
    let file = fs::File::open(PROCESS_FILENAME)?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(|line| SimProcess::try_from(line))
        .collect::<Result<Vec<SimProcess>>>()
}

pub fn display_processes(processes: &Vec<SimProcess>) {
    for process in processes {
        println!("{}", process);
    }
}

// algorithms

pub fn fcfs(mut incoming: Vec<SimProcess>) -> Vec<SimProcess> {
    let mut finished: Vec<SimProcess> = vec![];
    incoming.reverse();

    while incoming.len() != 0 {
        let mut process_current = incoming.pop().unwrap();
        for process_next in incoming.iter_mut() {
            process_next.wait += process_current.burst;
        }
        process_current.burst = 0;
        finished.push(process_current);
    }

    finished
}