//! Show metadata about histograms

extern crate histogram;

use histogram::{Type, Histogram};
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
    println!("Histogram type size: {}", mem::size_of::<Type>());
    println!("Histogram size: {}", mem::size_of::<Histogram>());

    Ok(())
}
