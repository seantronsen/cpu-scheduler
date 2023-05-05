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

use scheduler::{self, algo, sim, Configuration, ScheduleKind};
fn main() -> scheduler::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let config = match Configuration::build(&args) {
        scheduler::Result::Ok(config) => config,
        scheduler::Result::Err(e) => {
            eprintln!("error: {:?}", e);
            scheduler::print_usage_statement(args);
            std::process::exit(1);
        }
    };
    run(config)
}

fn run(config: Configuration) -> scheduler::Result<()> {
    let order = match config.scheduler {
        ScheduleKind::Priority | ScheduleKind::PriorityRR => sim::OrderKind::Priority,
        _ => sim::OrderKind::Burst,
    };

    let processes = scheduler::read_processes(order, &config.filename)?;
    println!("received: input processes");
    println!("process schedule kind: {:?}", config.scheduler);
    scheduler::display_processes(&processes);
    println!();

    let finished = match config.scheduler {
        ScheduleKind::FCFS => algo::fcfs(processes),
        ScheduleKind::SJF => algo::sort_before_fcfs(processes),
        ScheduleKind::Priority => algo::sort_before_fcfs(processes),
        ScheduleKind::RR => algo::round_robin(processes, 10),
        ScheduleKind::PriorityRR => todo!(), //algo::priority_rr(processes, 10),
    };
    scheduler::display_processes(&finished);
    Ok(())
}
