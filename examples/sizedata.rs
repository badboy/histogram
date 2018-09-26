//! Show metadata about histograms

extern crate histogram;

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
    println!("HistoType size: {}",  std::mem::size_of::<histogram::HistoTyp>());
    println!("Histogram size: {}",  std::mem::size_of::<histogram::Histogram>());

    Ok(())
}
