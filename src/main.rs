use std::{
    io::BufRead,
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        RwLock,
    },
};

use rayon::prelude::*;

// Tracks the sum of consecutive values in a dynamically sized array where the values can be written
// in any order.
struct LineCountState {
    counts: Box<[usize]>,
    next_index: usize,
    sum: usize,
}

impl LineCountState {
    fn new(len: usize) -> Self {
        Self {
            counts: vec![usize::MAX; len].into_boxed_slice(),
            next_index: 0,
            sum: 0,
        }
    }

    /// Sum of all consecutive counts.
    fn sum(&self) -> usize {
        self.sum
    }

    /// Write count at index.
    fn write(&mut self, index: usize, count: usize) {
        debug_assert!(
            self.counts[index] == usize::MAX,
            "second write to same index"
        );
        debug_assert!(count != usize::MAX, "count can not be usize::MAX");

        self.counts[index] = count;

        // Update sum and next index.
        while self.next_index < self.counts.len() {
            let count = self.counts[self.next_index];
            if count == usize::MAX {
                break;
            }
            self.sum += count;
            self.next_index += 1;
        }
    }
}

fn main() -> std::io::Result<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build_global()
        .unwrap();

    let paths = ["a.txt", "b.txt", "c.txt", "d.txt"]
        .into_iter()
        .map(Path::new)
        .collect::<Vec<_>>();

    let line_limit = 10;

    // Used to determine the next file to open. This guarantees the order.
    let path_index = AtomicUsize::new(0);
    let line_counts = RwLock::new(LineCountState::new(paths.len()));

    let mut index_and_lines = (0..paths.len())
        .into_par_iter()
        .map(|_| -> std::io::Result<(usize, Vec<String>)> {
            let index = path_index.fetch_add(1, Ordering::SeqCst);
            let path = paths[index];
            let prefix = format!("{}{index}{}", " ".repeat(index), " ".repeat(3 - index));
            let tid = std::thread::current().id();

            {
                let line_counts = line_counts.read().unwrap();
                let sum = line_counts.sum();
                let total: usize = line_counts
                    .counts
                    .iter()
                    .copied()
                    .filter(|&v| v != usize::MAX)
                    .sum();
                println!(
                    "{prefix} {tid:?}: sequential lines read = {sum}, total lines read = {total}",
                );

                if sum >= line_limit {
                    println!(
                        "{prefix} {tid:?}: skipping {:?} because we already have enough data",
                        path.display()
                    );

                    // Read enough, return empty
                    return Ok(Default::default());
                }
            }

            println!("{prefix} {tid:?}: opening {:?}", path.display());
            let reader = std::io::BufReader::new(std::fs::File::open(path)?);
            let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;
            // Pretend that the file is larger and that the time to reading and parse the data
            // depends on the file size.
            std::thread::sleep(std::time::Duration::from_millis(
                (100 * lines.len()).try_into().unwrap(),
            ));
            println!(
                "{prefix} {tid:?}: closing {:?} with {} lines read",
                path.display(),
                lines.len()
            );

            {
                let mut line_counts = line_counts.write().unwrap();
                line_counts.write(index, lines.len());
            }

            Ok((index, lines))
        })
        .collect::<std::io::Result<Vec<(usize, Vec<_>)>>>()?;

    index_and_lines.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

    let lines = index_and_lines
        .into_iter()
        .flat_map(|(_, lines)| lines)
        .collect::<Vec<_>>();

    println!("{lines:?}");

    Ok(())
}
