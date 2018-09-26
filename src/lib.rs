//! # A histogram library
//!
//! A simple library for histograms, with features required by Firefox.
//!
//! It supports linear, exponential, bool and enumerated histograms.
//! It can be serialized to a packed and a full representation.
//! It can be constructed from FFI-provided bucket boundaries,
//! avoiding additional allocation for metadata.
//!
//! ## Example
//!
//! ```rust
//! # use histogram::Histogram;
//! let mut hist = Histogram::exponential(1, 500, 10);
//!
//! for i in 1..=10 {
//!     hist.add(i);
//! }
//!
//! assert_eq!(10, hist.count());
//! assert_eq!(55, hist.sum());
//!
//! for bucket in hist.buckets() {
//!   println!("Bucket {}..{} has {} elements", bucket.start(), bucket.end(), bucket.count());
//! }
//! ```

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::cmp;
use std::collections::BTreeMap;
use std::fmt;

use serde::ser::{Serialize, SerializeStruct, Serializer};

pub mod ffi;

/// The type of a histogram.
#[derive(Debug, Serialize)]
pub enum Type {
    Linear,
    Exponential,
    Boolean,
    Enumerated,
}

/// A histogram.
///
/// Stores the ranges of buckets as well as counts per buckets.
/// It also tracks the count of added values and the total sum.
#[derive(Debug)]
pub struct Histogram {
    min: usize,
    max: usize,
    ranges: &'static [usize],
    buckets: Vec<usize>,

    count: usize,
    sum: usize,
    typ: Type,
}

fn linear_range(min: usize, max: usize, count: usize) -> Vec<usize> {
    let mut ranges = Vec::with_capacity(count);
    ranges.push(0);

    for i in 1..count {
        let range = (min * (count - 1 - i) + max * (i - 1)) / (count - 2);
        ranges.push(range);
    }

    ranges
}

fn exponential_range(min: usize, max: usize, count: usize) -> Vec<usize> {
    let log_max = (max as f64).ln();

    let mut ranges = Vec::with_capacity(count);
    ranges.push(0);
    let mut current = min;
    if current == 0 {
        current = 1;
    }
    ranges.push(current);

    for i in 2..count {
        let log_current = (current as f64).ln();
        let log_ratio = (log_max - log_current) / (count - i) as f64;
        let log_next = log_current + log_ratio;
        let next_value = log_next.exp().round() as usize;
        current = if next_value > current {
            next_value
        } else {
            current + 1
        };
        ranges.push(current);
    }

    ranges
}

fn pack_histogram(buckets: Buckets) -> Vec<(usize, usize)> {
    let mut res = vec![];

    let mut first = true;
    let mut last = 0;
    let len = buckets.histogram.bucket_count();
    let mut last_start = 42;
    let mut previous_start = 0;

    for (idx, bucket) in buckets.enumerate() {
        if bucket.count() == 0 {
            continue;
        }

        if idx > 0 && first {
            res.push((previous_start, 0))
        }
        last_start = bucket.end;
        first = false;
        last = idx + 1;
        previous_start = bucket.start;
        res.push((bucket.start, bucket.count));
    }

    if last > 0 && last < len {
        res.push((last_start, 0))
    }

    res
}

impl Histogram {
    /// Create a histogram with a range of min..max from the given ranges.
    ///
    /// ## Requirements
    ///
    /// * `ranges.len()` is the number of buckets
    pub fn factory_get(min: usize, max: usize, ranges: &'static [usize]) -> Histogram {
        Histogram {
            min,
            max,
            ranges,
            buckets: vec![0; ranges.len()],
            count: 0,
            sum: 0,
            typ: Type::Linear,
        }
    }

    /// Create a histogram with `count` linear  buckets in the range `min` to `max`.
    ///
    /// The minimum will be at least 1.
    pub fn linear(min: usize, max: usize, count: usize) -> Histogram {
        let min = cmp::max(1, min);

        let ranges = linear_range(min, max, count);
        let ranges = Box::leak(ranges.into_boxed_slice());

        Histogram {
            min,
            max,
            ranges,
            buckets: vec![0; count],
            count: 0,
            sum: 0,
            typ: Type::Linear,
        }
    }

    /// Create a histogram with `count` exponential buckets in the range `min` to `max`.
    ///
    /// The minimum will be at least 1.
    pub fn exponential(min: usize, max: usize, count: usize) -> Histogram {
        let min = cmp::max(1, min);

        let ranges = exponential_range(min, max, count);
        let ranges = Box::leak(ranges.into_boxed_slice());

        Histogram {
            min,
            max,
            ranges,
            buckets: vec![0; count],
            count: 0,
            sum: 0,
            typ: Type::Exponential,
        }
    }

    /// Create a flag histogram.
    ///
    /// This histogram type allows you to record a single value (0 or 1, default 0).
    ///
    /// **Deprecated.**
    pub fn flag() -> Histogram {
        Self::boolean()
    }

    /// Create a boolean histogram.
    ///
    /// These histograms only record boolean values.
    pub fn boolean() -> Histogram {
        let mut h = Self::linear(1, 2, 3);
        h.typ = Type::Boolean;
        h
    }

    /// Create a histogram over enumeratable values.
    ///
    /// An enumerated histogram consists of exactly `count` buckets.
    /// Each bucket is associated with a consecutive integer.
    pub fn enumerated(count: usize) -> Histogram {
        let mut h = Self::linear(1, count, count + 1);
        h.typ = Type::Enumerated;
        h
    }

    /// Get the number of buckets in this histogram.
    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    /// Add a single value to this histogram.
    pub fn add(&mut self, value: usize) {
        self.accumulate(value, 1);
    }

    /// Add `count` number of values.
    pub fn accumulate(&mut self, value: usize, count: usize) {
        self.sum += value * count;
        self.count += count;
        *self.bucket(value) += 1;
    }

    /// Get an iterator over this histogram's buckets.
    pub fn buckets(&self) -> Buckets {
        Buckets {
            histogram: self,
            index: 0,
        }
    }

    /// Get the total sum of values recorded in this histogram.
    pub fn sum(&self) -> usize {
        self.sum
    }

    /// Get the total count of values recorded in this histogram.
    pub fn count(&self) -> usize {
        self.count
    }

    fn bucket(&mut self, value: usize) -> &mut usize {
        let mut under = 0;
        let mut over = self.bucket_count();
        let mut mid;

        loop {
            mid = under + (over - under) / 2;
            if mid == under {
                break;
            }
            if self.ranges[mid] <= value {
                under = mid;
            } else {
                over = mid;
            }
        }

        &mut self.buckets[mid]
    }

    /// Get a packed representation of this histogram.
    pub fn persisted(&self) -> PackedHistogram {
        PackedHistogram { histogram: self }
    }
}

/// An iterator over the buckets in a histogram.
#[derive(Debug, Clone)]
pub struct Buckets<'a> {
    histogram: &'a Histogram,
    index: usize,
}

impl<'a> Iterator for Buckets<'a> {
    type Item = Bucket;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.histogram.bucket_count() {
            return None;
        }
        let start = self.histogram.ranges[self.index];
        let end = if self.index + 1 == self.histogram.bucket_count() {
            ::std::usize::MAX
        } else {
            self.histogram.ranges[self.index + 1]
        };

        let count = self.histogram.buckets[self.index];
        self.index += 1;

        Some(Bucket { start, end, count })
    }
}

/// A bucket is a range of samples and their count.
#[derive(Clone)]
pub struct Bucket {
    start: usize,
    end: usize,
    count: usize,
}

impl Bucket {
    /// The number of samples in this bucket's range.
    pub fn count(&self) -> usize {
        self.count
    }

    /// The start of this bucket's range.
    pub fn start(&self) -> usize {
        self.start
    }

    /// The end of this bucket's range.
    pub fn end(&self) -> usize {
        self.end
    }
}

impl fmt::Debug for Bucket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bucket {{ {}..{} }}", self.start, self.end)
    }
}

impl fmt::Display for Histogram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Write;

        let count = self.count;
        writeln!(f, "# Number of samples = {}", count)?;
        if count == 0 {
            return Ok(());
        }

        let max_bucket_count = self.buckets().map(|b| b.count()).fold(0, cmp::max);

        const WIDTH: usize = 50;
        let count_per_char = cmp::max(max_bucket_count / WIDTH, 1);

        writeln!(f, "# Each ∎ is a count of {}", count_per_char)?;
        writeln!(f, "#")?;

        let mut count_str = String::new();

        let widest_count = self.buckets().fold(0, |n, b| {
            count_str.clear();
            write!(&mut count_str, "{}", b.count()).unwrap();
            cmp::max(n, count_str.len())
        });

        let mut end_str = String::new();
        let widest_range = self.buckets().fold(0, |n, b| {
            end_str.clear();
            if b.end() == ::std::usize::MAX {
                cmp::max(n, 3)
            } else {
                write!(&mut end_str, "{}", b.end()).unwrap();
                cmp::max(n, end_str.len())
            }
        });

        let mut start_str = String::with_capacity(widest_range);

        for bucket in self.buckets() {
            start_str.clear();
            write!(&mut start_str, "{}", bucket.start()).unwrap();
            for _ in 0..widest_range - start_str.len() {
                start_str.insert(0, ' ');
            }

            end_str.clear();
            if bucket.end() == ::std::usize::MAX {
                write!(&mut end_str, "INF").unwrap();
            } else {
                write!(&mut end_str, "{}", bucket.end()).unwrap();
            }
            for _ in 0..widest_range - end_str.len() {
                end_str.insert(0, ' ');
            }

            count_str.clear();
            write!(&mut count_str, "{}", bucket.count()).unwrap();
            for _ in 0..widest_count - count_str.len() {
                count_str.insert(0, ' ');
            }

            write!(f, "{} .. {} [ {} ]: ", start_str, end_str, count_str)?;
            for _ in 0..bucket.count() / count_per_char {
                write!(f, "∎")?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

/// Packed representation of a histogram for serialization
pub struct PackedHistogram<'a> {
    histogram: &'a Histogram,
}

impl<'a> Serialize for PackedHistogram<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PackedHistogram", 2)?;
        state.serialize_field("sum", &self.histogram.sum)?;
        state.serialize_field("counts", &self.histogram.buckets)?;
        state.end()
    }
}

impl Serialize for Histogram {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Histogram", 5)?;
        state.serialize_field("range", &[self.min, self.max])?;
        state.serialize_field("bucket_count", &self.bucket_count())?;
        state.serialize_field("histogram_type", &self.typ)?;
        let values = pack_histogram(self.buckets())
            .iter()
            .map(|&(a, b)| (a.to_string(), b))
            .collect::<BTreeMap<String, usize>>();
        state.serialize_field("values", &values)?;
        state.serialize_field("sum", &self.sum)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear() {
        let mut h = Histogram::linear(1, 500, 10);

        h.add(0);
        h.add(1);
        h.add(14);
        h.add(450);
        h.add(700);

        assert_eq!(5, h.count());
        assert_eq!(0 + 1 + 14 + 450 + 700, h.sum());

        let expected_counts = [1, 2, 0, 0, 0, 0, 0, 0, 1, 1];
        for (bucket, &expected) in h.buckets().zip(expected_counts.iter()) {
            assert_eq!(
                expected,
                bucket.count(),
                "{:?} should have {} values",
                bucket,
                expected
            );
        }
    }

    #[test]
    fn exp() {
        let mut h = Histogram::exponential(1, 500, 10);

        h.add(0);
        h.add(1);
        h.add(14);
        h.add(450);
        h.add(700);

        assert_eq!(5, h.count());
        assert_eq!(0 + 1 + 14 + 450 + 700, h.sum());

        let expected_counts = [1, 1, 0, 0, 1, 0, 0, 0, 1, 1];
        for (bucket, &expected) in h.buckets().zip(expected_counts.iter()) {
            assert_eq!(
                expected,
                bucket.count(),
                "{:?} should have {} values",
                bucket,
                expected
            );
        }
    }

    #[test]
    fn enumerated() {
        let mut h = Histogram::enumerated(10);

        for i in 0..10 {
            h.add(i + 1);
        }
        h.add(10);

        assert_eq!(11, h.count());
        assert_eq!(10 + 10 + 9 + 8 + 7 + 6 + 5 + 4 + 3 + 2 + 1, h.sum());

        let expected_counts = [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2];
        for (bucket, &expected) in h.buckets().zip(expected_counts.iter()) {
            assert_eq!(
                expected,
                bucket.count(),
                "{:?} should have {} values",
                bucket,
                expected
            );
        }
    }

    #[test]
    fn boolean() {
        let mut h = Histogram::boolean();

        for i in 0..10 {
            h.add((i % 2 == 0) as usize);
        }

        assert_eq!(10, h.count());
        assert_eq!(5, h.sum());

        let expected_counts = [5, 5, 0];
        for (bucket, &expected) in h.buckets().zip(expected_counts.iter()) {
            assert_eq!(
                expected,
                bucket.count(),
                "{:?} should have {} values",
                bucket,
                expected
            );
        }
    }
}
