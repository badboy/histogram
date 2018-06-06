//! Reads samples from stdin, one per line, and then prints the resulting
//! histogram.

extern crate histogram;

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
    let mut hist = histogram::Histogram::exp_with_bucket_count(10, 1, 500);

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
    Ok(())
}
