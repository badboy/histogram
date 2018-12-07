//! Create, modify and serialize histograms over FFI.

use serde_json;
use std::ffi::CString;
use std::os::raw::{c_char, c_uint};
use std::slice;

use super::Histogram;

/// A histogram created from static data for ranges.
pub type StaticHistogram = Histogram<&'static [u32]>;

/// Create a new histogram from an external array of ranges.
#[no_mangle]
pub unsafe extern "C" fn histogram_factory_get(
    min: c_uint,
    max: c_uint,
    bucket_count: usize,
    ranges: *const c_uint,
) -> *mut StaticHistogram {
    let ranges: &'static [u32] = slice::from_raw_parts(ranges, bucket_count + 1);
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
    histogram.add(sample as usize);
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
