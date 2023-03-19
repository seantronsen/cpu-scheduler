use crate::sim::SimProcess;
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


