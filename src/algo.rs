use crate::sim::SimProcess;
use crate::structures::DLL;
pub fn fcfs(mut incoming: Vec<SimProcess>) -> Vec<SimProcess> {
    let mut finished: Vec<SimProcess> = vec![];
    incoming.reverse();

    let mut running_time = 0;

    while incoming.len() != 0 {
        let mut process_current = incoming.pop().unwrap();
        let burst_time = process_current.remaining_burst;
        process_current.run_burst(running_time, burst_time);
        finished.push(process_current);
        running_time += burst_time;
    }

    finished
}

fn mergesort(collection: Vec<SimProcess>) -> Vec<SimProcess> {
    let length = collection.len();
    if length == 0 {
        return collection;
    } else {
        let mut collection: Vec<Option<SimProcess>> =
            collection.into_iter().map(|x| Some(x)).collect();
        thunk_mergesort(&mut collection[..], length)
            .into_iter()
            .map(|x| x.unwrap())
            .collect::<Vec<SimProcess>>()
    }
}

fn thunk_mergesort(
    collection: &mut [Option<SimProcess>],
    length: usize,
) -> Vec<Option<SimProcess>> {
    let mut result = Vec::with_capacity(length);
    // do nothing, already sorted
    if length == 1 {
        result.push(collection[0].take());
        result
    } else {
        let len_a = length / 2;
        let len_b = length - len_a;
        let (collection_a, collection_b) = collection.split_at_mut(len_a);
        let mut collection_a = thunk_mergesort(collection_a, len_a);
        let mut collection_b = thunk_mergesort(collection_b, len_b);

        // merge
        let mut index_a = 0;
        let mut index_b = 0;

        for _ in 0..length {
            if index_a < len_a
                && (index_b == len_b || &collection_a[index_a] <= &collection_b[index_b])
            {
                result.push(collection_a[index_a].take());
                index_a += 1;
            } else if index_b < len_b
                && (index_a == len_a || &collection_b[index_b] <= &collection_a[index_a])
            {
                result.push(collection_b[index_b].take());
                index_b += 1;
            }
        }

        result
    }
}

/// using the rules applied via the struct and mergesort implementation, fill algorithmic
/// requirements by applying a simple sort and transitioning to the standard FCFS approach.
pub fn sort_before_fcfs(incoming: Vec<SimProcess>) -> Vec<SimProcess> {
    fcfs(mergesort(incoming))
}

pub fn round_robin(incoming: Vec<SimProcess>, quantum: u32) -> Vec<SimProcess> {
    let mut outgoing: DLL<SimProcess> = DLL::from(vec![]);
    let mut incoming: DLL<SimProcess> = DLL::from(incoming);
    let mut current_time: u32 = 0;

    while !incoming.is_empty() {
        if let Some(mut current_process) = incoming.pop_front() {
            let mut burst = quantum;
            let mut destination = Some(&mut incoming);
            if current_process.remaining_burst > quantum {
                current_process.run_burst(current_time, burst);
            } else {
                destination.replace(&mut outgoing);
                burst -= burst - current_process.remaining_burst;
                current_process.run_burst(current_time, burst);
            }

            current_time += burst;
            println!(
                "Time: {} | Burst Complete for {}",
                &current_time, &current_process
            );

            destination.unwrap().append(current_process);
        }
    }

    outgoing.into()
}

pub fn priority_rr(incoming: Vec<SimProcess>, quantum: u32) -> Vec<SimProcess> {
    let mut incoming = DLL::from(mergesort(incoming));
    let mut outgoing: DLL<SimProcess> = DLL::new();
    let mut current_time: u32 = 0;

    while let Some(mut process) = incoming.pop_front() {
        let burst = match quantum <= process.remaining_burst {
            true => quantum,
            false => quantum - (process.remaining_burst % quantum),
        };

        process.run_burst(current_time, burst);
        current_time += burst;

        if process.remaining_burst == 0 {
            outgoing.append(process);
        } else {
            match incoming.iter().position(|x| x.priority > process.priority) {
                Some(index) => incoming.insert(index, process),
                None => incoming.append(process),
            }
        }
    }

    outgoing.into()
}
