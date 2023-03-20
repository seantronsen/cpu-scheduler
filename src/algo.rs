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

// sorting for sjf and so on

fn real_mergesort(mut collection: Vec<SimProcess>) -> Vec<SimProcess> {
    let length = collection.len();
    if length == 0 {
        return collection;
    } else {
        let mut space: Vec<Option<SimProcess>> = Vec::with_capacity(length);
        space.fill_with(|| None);
        let mut collection: Vec<Option<SimProcess>> =
            collection.into_iter().map(|x| Some(x)).collect();
        hidden_mergesort(&mut collection[..], &mut space[..], length);
        collection
            .into_iter()
            .map(|x| x.unwrap())
            .collect::<Vec<SimProcess>>()
    }
}

fn hidden_mergesort(
    collection: &mut [Option<SimProcess>],
    space: &mut [Option<SimProcess>],
    length: usize,
) {
    if length == 1 {
        // do nothing, already sorted
        return;
    } else {
        let partition = length / 2;
        let (collection_a, collection_b) = collection.split_at_mut(partition);
        let (space_a, space_b) = space.split_at_mut(partition);
        hidden_mergesort(collection_a, space_a, partition);
        hidden_mergesort(collection_b, space_b, length - partition);

        // merge

        for i in 0..length {}
    }
}
