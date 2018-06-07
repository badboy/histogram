//! Reads samples from stdin, one per line, and then prints the resulting
//! histogram.

extern crate histogram;
extern crate serde;
extern crate serde_json;

use std::io::{self, BufRead, Write};
use std::process;

fn main() {
    if let Err(e) = try_main() {
        let mut stderr = io::stderr();
        let _ = write!(&mut stderr, "error: {}", e);
        process::exit(1);
    }
}

fn try_main() -> io::Result<()> {
    let mut hist = histogram::Histogram::exponential(10, 1, 500);

    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let mut line = String::new();
    while stdin.read_line(&mut line)? > 0 {
        {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let sample: usize = line.trim()
                .parse()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            hist.add(sample);
        }

        line.clear();
    }

    println!("{}", hist);
    let serialized = serde_json::to_string(&hist.persisted()).unwrap();
    println!("persisted = {}", serialized);

    let serialized = serde_json::to_string_pretty(&hist).unwrap();
    println!("serialized = \n{}", serialized);
    Ok(())
}
