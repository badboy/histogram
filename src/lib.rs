extern crate serde;

use std::cmp;
use std::fmt;
use std::collections::HashMap;

use serde::ser::{Serialize, Serializer, SerializeStruct};

#[derive(Debug)]
pub struct Histogram {
    bucket_count: usize,
    min: usize,
    max: usize,
    ranges: Vec<usize>,
    buckets: Vec<usize>,

    count: usize,
    sum: usize,
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

    for i in 2 .. count {
        let log_current = (current as f64).ln();
        let log_ratio = (log_max - log_current) / (count - i) as f64;
        let log_next = log_current + log_ratio;
        let next_value = log_next.exp().round() as usize;
        current = if next_value > current { next_value } else { current + 1 };
        ranges.push(current);
    }

  ranges
}


impl Histogram {
    pub fn with_bucket_count(count: usize, min: usize, max: usize) -> Histogram {
        let min = cmp::max(1, min);

        let ranges = linear_range(min, max, count);

        Histogram {
            bucket_count: count,
            min: 0,
            max: max,
            ranges: ranges,
            buckets: vec![0; count],
            count: 0,
            sum: 0,
        }
    }

    pub fn exp_with_bucket_count(count: usize, min: usize, max: usize) -> Histogram {
        let min = cmp::max(1, min);

        let ranges = exponential_range(min, max, count);

        Histogram {
            bucket_count: count,
            min: 0,
            max: max,
            ranges: ranges,
            buckets: vec![0; count],
            count: 0,
            sum: 0,
        }
    }

    pub fn add(&mut self, value: usize) {
        self.accumulate(value, 1);
    }

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

    fn bucket(&mut self, value: usize) -> &mut usize {
        let mut under = 0;
        let mut over = self.bucket_count;
        let mut mid;

        loop {
            mid = under + (over - under)/2;
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

    pub fn persisted(&self) -> PersistedHistogram {
        PersistedHistogram {
            histogram: self
        }
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
        if self.index >= self.histogram.bucket_count {
            return None;
        }
        let start = self.histogram.ranges[self.index];
        let end = if self.index+1 == self.histogram.bucket_count {
            ::std::usize::MAX
        } else {
            self.histogram.ranges[self.index+1]
        };

        let count = self.histogram.buckets[self.index];
        self.index += 1;

        Some(Bucket {
            start,
            end,
            count: count,
        })
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

pub struct PersistedHistogram<'a> {
    histogram: &'a Histogram
}

impl<'a> Serialize for PersistedHistogram<'a>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PersistedHistogram", 2)?;
        state.serialize_field("sum", &self.histogram.sum)?;
        state.serialize_field("counts", &self.histogram.buckets)?;
        state.end()
    }
}

impl Serialize for Histogram
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Histogram", 5)?;
        state.serialize_field("range", &self.ranges)?;
        state.serialize_field("bucket_count", &self.bucket_count)?;
        state.serialize_field("histogram_type", &1)?;
        let values = self.buckets.iter().enumerate().map(|(idx, count)| {
            (self.ranges[idx].to_string(), *count)
        }).collect::<HashMap<String, usize>>();
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
        let mut h = Histogram::with_bucket_count(10, 1, 500);
        println!("{:#?}", h);

        h.add(0);
        h.add(1);
        h.add(14);
        h.add(450);
        h.add(700);

        println!("{:#?}", h);

        assert!(false);
    }

    #[test]
    fn exp() {
        let mut h = Histogram::exp_with_bucket_count(10, 1, 500);
        println!("{:#?}", h);

        h.add(0);
        h.add(1);
        h.add(14);
        h.add(450);
        h.add(700);

        println!("{:#?}", h);

        assert!(false);
    }
}
