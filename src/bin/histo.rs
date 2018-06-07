//! Reads samples from stdin, one per line, and then prints the resulting
//! histogram.

extern crate histogram;
extern crate serde;
extern crate serde_json;

use std::env;
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
    let mut args = env::args().skip(1);
    let typ = args.next().unwrap_or_else(|| String::from("linear"));
    let min = args.next().unwrap_or_else(|| String::from("1")).parse::<usize>().unwrap();
    let max = args.next().unwrap_or_else(|| String::from("500")).parse::<usize>().unwrap();
    let count = args.next().unwrap_or_else(|| String::from("10")).parse::<usize>().unwrap();

    let mut hist = if typ == "linear" {
        histogram::Histogram::linear(min, max, count)
    } else {
        histogram::Histogram::exponential(min, max, count)
    };

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
