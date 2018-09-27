//! Generate a random histogram and display it JSON seralized.

extern crate histogram;
extern crate rand;
extern crate serde;
extern crate serde_json;

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
    let count = 10;
    let mut hist = histogram::Histogram::exponential(min, max, count);

    let sample_count = 1_000;
    for _ in 0..sample_count {
        let sample = rng.gen_range(min, max + 1);
        hist.add(sample);
    }

    let serialized = serde_json::to_string(&hist.persisted()).unwrap();
    println!("persisted = {}", serialized);

    let serialized = serde_json::to_string_pretty(&hist).unwrap();
    println!("serialized = \n{}", serialized);

    Ok(())
}
