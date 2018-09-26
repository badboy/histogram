//! Show metadata about histograms

extern crate histogram;

use histogram::{HistoTyp, Histogram};
use std::io::{self, Write};
use std::{mem, process};

fn main() {
    if let Err(e) = try_main() {
        let mut stderr = io::stderr();
        let _ = write!(&mut stderr, "error: {}", e);
        process::exit(1);
    }
}

fn try_main() -> io::Result<()> {
    println!("HistoType size: {}", mem::size_of::<HistoTyp>());
    println!("Histogram size: {}", mem::size_of::<Histogram>());

    Ok(())
}
