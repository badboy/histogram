//! Create, modify and serialize histograms over FFI.

use serde_json;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uint};
use std::slice;

use super::Histogram;
use super::Snapshot;

/// A histogram created from static data for ranges.
pub type StaticHistogram = Histogram<&'static [u32]>;

/// Create a new histogram from an external array of ranges.
#[no_mangle]
pub unsafe extern "C" fn histogram_factory_get(
    min: c_uint,
    max: c_uint,
    bucket_count: usize,
    ranges: *const c_int,
) -> *mut StaticHistogram {
    let ranges: &'static [u32] = slice::from_raw_parts(ranges as *const c_uint, bucket_count + 1);
    assert_eq!(::std::i32::MAX, ranges[bucket_count] as i32);
    let h = Histogram {
        min: min,
        max: max,
        ranges,
        buckets: vec![0; bucket_count as usize].into_boxed_slice(),
        count: 0,
        sum: 0,
        typ: super::Type::External,
    };

    Box::into_raw(Box::new(h))
}

/// Free a histogram's memory.
#[no_mangle]
pub unsafe extern "C" fn histogram_free(histogram: *mut StaticHistogram) {
    let _ = Box::from_raw(histogram);
}

/// Add a single value to the given histogram.
#[no_mangle]
pub unsafe extern "C" fn histogram_add(histogram: *mut StaticHistogram, sample: c_uint) {
    let histogram = &mut *histogram;
    histogram.add(sample);
}

/// Clear the stored data in the histogram
#[no_mangle]
pub unsafe extern "C" fn histogram_clear(histogram: *mut StaticHistogram) {
    let histogram = &mut *histogram;
    histogram.clear();
}

/// Check if this histogram recorded any values.
#[no_mangle]
pub unsafe extern "C" fn histogram_is_empty(histogram: *const StaticHistogram) -> bool {
    let histogram = &*histogram;
    histogram.is_empty()
}

/// Get the number of buckets in this histogram.
#[no_mangle]
pub unsafe extern "C" fn histogram_bucket_count(histogram: *const StaticHistogram) -> usize {
    let histogram = &*histogram;
    histogram.bucket_count()
}

#[no_mangle]
pub unsafe extern "C" fn histogram_ranges(histogram: *const StaticHistogram, idx: c_int) -> u32 {
    let histogram = &*histogram;
    histogram.ranges[idx as usize]
}

/// Serialize the histogram into a persistable JSON string.
///
/// The returned data is null-terminated. It should be passed back to `histogram_free_cstr` to
/// deallocate after usage.
#[no_mangle]
pub unsafe extern "C" fn histogram_serialize_persist(histogram: *mut StaticHistogram) -> *mut c_char {
    let histogram = &*histogram;
    let serialized = serde_json::to_string(&histogram.persisted()).unwrap();
    CString::new(serialized.to_string()).unwrap().into_raw()
}

/// Serialize the histogram into a packed representation.
///
/// The returned data is null-terminated. It should be passed back to `histogram_free_cstr` to
/// deallocate after usage.
#[no_mangle]
pub unsafe extern "C" fn histogram_serialize(histogram: *mut StaticHistogram) -> *mut c_char {
    let serialized = serde_json::to_string(&*histogram).unwrap();
    CString::new(serialized.to_string()).unwrap().into_raw()
}

/// Deallocate a null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn histogram_free_cstr(s: *mut c_char) {
    let _str = CString::from_raw(s);
}

#[no_mangle]
pub unsafe extern "C" fn histogram_snapshot(histogram: *const StaticHistogram) -> *mut Snapshot {
    let histogram = &*histogram;
    Box::into_raw(Box::new(histogram.snapshot()))
}

#[no_mangle]
pub unsafe extern "C" fn histogram_snapshot_counts(snapshot: *const Snapshot, idx: c_int) -> u32 {
    let snapshot = &*snapshot;
    snapshot.counts[idx as usize]
}

#[no_mangle]
pub unsafe extern "C" fn histogram_snapshot_sum(snapshot: *const Snapshot) -> u32 {
    let snapshot = &*snapshot;
    snapshot.sum
}

#[no_mangle]
pub unsafe extern "C" fn histogram_snapshot_free(snapshot: *mut Snapshot) {
    let _ = Box::from_raw(snapshot);
}
