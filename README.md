[![Build Status](https://travis-ci.org/badboy/histogram.svg?branch=master)](https://travis-ci.org/badboy/histogram)

# histogram

## A histogram library

A simple library for histograms, with features required by Firefox.

It supports linear, exponential, bool and enumerated histograms.
It can be serialized to a packed and a full representation.
It can be constructed from FFI-provided bucket boundaries,
avoiding additional allocation for metadata.

### Example

```rust
let mut hist = Histogram::exponential(1, 500, 10);

for i in 1..=10 {
    hist.add(i);
}

assert_eq!(10, hist.count());
assert_eq!(55, hist.sum());

for bucket in hist.buckets() {
    println!("Bucket {}..{} has {} elements", bucket.start(), bucket.end(), bucket.count());
}
```

## License

MIT. See [LICENSE](LICENSE).
