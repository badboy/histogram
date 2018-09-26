use serde_json;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::slice;

use super::Histogram;

#[no_mangle]
pub unsafe extern "C" fn histogram_factory_get(
    min: c_int,
    max: c_int,
    bucket_count: usize,
    ranges: *const usize,
) -> *mut Histogram {
    let ranges: &'static [usize] = slice::from_raw_parts(ranges, bucket_count + 1);
    assert_eq!(::std::i32::MAX, ranges[bucket_count] as i32);
    let h = Histogram {
        min: min as usize,
        max: max as usize,
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
pub unsafe extern "C" fn histogram_add(histogram: *mut Histogram, value: c_int) {
    debug_assert!(!histogram.is_null());
    let histogram = &mut *histogram;
    histogram.add(value as usize);
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
