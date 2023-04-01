/*
Author: Sean Tronsen
Programming Project: Scheduling Algorithms
Chapter: 05

Prompt: This project involves implementing several different process scheduling algorithms. The
scheduler will be assigned a predefined set of tasks and will schedule the tasks based on the
selected scheduling algorithm. Each task is assigned a priority and CPU burst. The following
scheduling algorithms will be implemented:

• First-come, first-served (FCFS), which schedules tasks in the order in which they request the
    CPU.
• Shortest-job-first (SJF), which schedules tasks in order of the length of the tasks’ next CPU
    burst.
• Priority scheduling, which schedules tasks based on priority.
• Round-robin (RR) scheduling, where each task is run for a time quantum (or for the remainder of
    its CPU burst).
• Priority with round-robin, which schedules tasks in order of priority and uses round-robin
    scheduling for tasks with equal priority.

Priorities range from 1 to 10, where a higher numeric value indicates a higher relative priority.
For round-robin scheduling, the length of a time quantum is 10 milliseconds.
 */

use scheduler::{self, algo, sim};
fn main() -> scheduler::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let config = match Config::build(&args) {
        scheduler::Result::Ok(config) => config,
        scheduler::Result::Err(e) => {
            eprintln!("error: {:?}", e);
            print_usage_statement(args);
            std::process::exit(1);
        }
    };
    run(config)
}

enum Scheduler {
    FCFS,
    SJF,
    Priority,
    RR,
    PriorityRR,
}

struct Config {
    scheduler: Scheduler,
    in_filename: String,
}

impl Config {
    fn build(args: &Vec<String>) -> scheduler::Result<Self> {
        let count = args.len();
        let mut iter = args.into_iter();

        if count != 3 {
            return Err(scheduler::ProgramError::InvalidCommandInput);
        }

        // skip program name
        iter.next();

        let scheduler = match iter
            .next()
            .expect("<scheduler-type> is a required argument")
            .parse::<u8>()?
        {
            0 => Scheduler::FCFS,
            1 => Scheduler::SJF,
            2 => Scheduler::Priority,
            3 => Scheduler::RR,
            4 => Scheduler::PriorityRR,
            _ => return Err(scheduler::ProgramError::InvalidCommandInput),
        };

        let in_filename = match iter.next() {
            Some(str) => String::from(str),
            None => {
                eprintln!("<process-filename> is a required argument");
                return Err(scheduler::ProgramError::InvalidCommandInput);
            }
        };

        Ok(Self {
            scheduler,
            in_filename,
        })
    }
}

fn print_usage_statement(args: Vec<String>) {
    println!("usage: {} <scheduler-type> <process-filename>", args[0]);
    println!("received: {:?}", args);
}

fn run(config: Config) -> scheduler::Result<()> {
    let order = match config.scheduler {
        Scheduler::Priority => sim::OrderKind::Priority,
        Scheduler::PriorityRR => sim::OrderKind::Priority,
        _ => sim::OrderKind::Burst,
    };

    let processes = scheduler::read_processes(order)?;
    println!("received: input processes");
    scheduler::display_processes(&processes);
    println!();

    let finished = match config.scheduler {
        Scheduler::FCFS => algo::fcfs(processes),
        Scheduler::SJF => algo::sjf(processes),
        Scheduler::Priority => algo::priority(processes),
        _ => panic!("remaining cases are still in the todo phase"),
    };
    scheduler::display_processes(&finished);
    Ok(())
}
