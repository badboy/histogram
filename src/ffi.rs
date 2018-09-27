//! Create, modify and serialize histograms over FFI

use serde_json;
use std::ffi::CString;
use std::os::raw::{c_char, c_uint};
use std::slice;

use super::Histogram;

#[no_mangle]
pub unsafe extern "C" fn histogram_factory_get(
    min: c_uint,
    max: c_uint,
    bucket_count: c_uint,
    ranges: *const c_uint,
) -> *mut Histogram {
    let ranges: &'static [u32] = slice::from_raw_parts(ranges, bucket_count as usize + 1);
    assert_eq!(::std::i32::MAX, ranges[bucket_count as usize] as i32);
    let h = Histogram {
        min: min,
        max: max,
        ranges,
        buckets: vec![0; bucket_count as usize],
        count: 0,
        sum: 0,
        typ: super::Type::Linear,
    };

    Box::into_raw(Box::new(h))
}

#[no_mangle]
pub unsafe extern "C" fn histogram_free(ranges: *mut Histogram) {
    let _box = Box::from_raw(ranges);
}

#[no_mangle]
pub unsafe extern "C" fn histogram_add(histogram: *mut Histogram, value: c_uint) {
    debug_assert!(!histogram.is_null());
    let histogram = &mut *histogram;
    histogram.add(value as u32);
}

#[no_mangle]
pub unsafe extern "C" fn histogram_serialize_persist(histogram: *mut Histogram) -> *mut c_char {
    debug_assert!(!histogram.is_null());
    let histogram = &mut *histogram;

    let serialized = serde_json::to_string(&histogram.persisted()).unwrap();
    CString::new(serialized.to_string()).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn histogram_serialize(histogram: *mut Histogram) -> *mut c_char {
    debug_assert!(!histogram.is_null());
    let histogram = &mut *histogram;

    let serialized = serde_json::to_string(&histogram).unwrap();
    CString::new(serialized.to_string()).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn histogram_free_cstr(s: *mut c_char) {
    let _str = CString::from_raw(s);
}
