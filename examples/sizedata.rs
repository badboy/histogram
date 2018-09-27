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
    println!("Histogram<&'static [u32]> size: {}", mem::size_of::<Histogram<&'static [u32]>>());
    println!("Histogram<Vec> size: {}", mem::size_of::<Histogram<Vec<u32>>>());
    println!("Histogram<Box<[u32]>> size: {}", mem::size_of::<Histogram<Box<[u32]>>>());
    let linear = Histogram::linear(1,2,3);
    println!("Histogram.linear size: {}", mem::size_of_val(&linear));

    println!("Box<Histogram<&'static [u32]>> size: {}", mem::size_of::<Box<Histogram<&'static [u32]>>>());
    println!("Box<Histogram<Vec>> size: {}", mem::size_of::<Box<Histogram<Vec<u32>>>>());
    println!("Box<Histogram<Box<[u32]>>> size: {}", mem::size_of::<Box<Histogram<Box<[u32]>>>>());

    Ok(())
}
