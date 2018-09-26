//! Generate a random histogram and display it.

extern crate histogram;
extern crate rand;

use rand::{thread_rng, Rng};
use std::io::{self, Write};
use std::process;

fn main() {
    if let Err(e) = try_main() {
        let mut stderr = io::stderr();
        let _ = write!(&mut stderr, "error: {}", e);
        process::exit(1);
    }
}

fn try_main() -> io::Result<()> {
    let mut rng = thread_rng();
    let min = 0;
    let max = 500;
    let count = 50;
    let mut hist = histogram::Histogram::exponential(min, max, count);

    let sample_count = 1_000_000;
    for _ in 0..sample_count {
        let sample = rng.gen_range(min, max + 1);
        hist.add(sample);
    }

    println!("{}", hist);
    Ok(())
}
