use crate::mem::{ByteArray, String as InkoString};
use crate::result::Result as InkoResult;
use crate::state::State;
use std::cmp::min;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;
use std::str;
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

#[no_mangle]
pub unsafe extern "system" fn inko_string_new(
    state: *const State,
    bytes: *const u8,
    length: usize,
) -> *const InkoString {
    let bytes = slice::from_raw_parts(bytes, length).to_vec();
    let string = String::from_utf8_unchecked(bytes);

    InkoString::alloc((*state).string_class, string)
}

#[no_mangle]
pub unsafe extern "system" fn inko_string_concat(
    state: *const State,
    strings: *const *const InkoString,
    length: i64,
) -> *const InkoString {
    let slice = slice::from_raw_parts(strings, length as usize);
    let mut buffer = String::new();

    for &val in slice {
        buffer.push_str(InkoString::read(val));
    }

    InkoString::alloc((*state).string_class, buffer)
}

#[no_mangle]
pub unsafe extern "system" fn inko_string_drop(pointer: *const InkoString) {
    InkoString::drop(pointer);
}

#[no_mangle]
pub unsafe extern "system" fn inko_string_to_lower(
    state: *const State,
    string: *const InkoString,
) -> *const InkoString {
    InkoString::alloc(
        (*state).string_class,
        InkoString::read(string).to_lowercase(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn inko_string_to_upper(
    state: *const State,
    string: *const InkoString,
) -> *const InkoString {
    InkoString::alloc(
        (*state).string_class,
        InkoString::read(string).to_uppercase(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn inko_string_to_byte_array(
    state: *const State,
    string: *const InkoString,
) -> *mut ByteArray {
    let bytes = InkoString::read(string).as_bytes().to_vec();

    ByteArray::alloc((*state).byte_array_class, bytes)
}

#[no_mangle]
pub unsafe extern "system" fn inko_string_to_float(
    bytes: *mut u8,
    size: i64,
) -> InkoResult {
    // On paper this relies on undefined behaviour, should the slice cover a
    // range bytes that's not valid UTF-8. But in reality this shouldn't pose a
    // problem because:
    //
    // 1. The float parser almost immediately converts the `&str` to `&[u8]`
    //    then operates on that, not caring about the encoding.
    // 2. Simply storing invalid UTF-8 in a `&str` isn't undefined behaviour
    //    (see https://github.com/rust-lang/rust/issues/71033), but using
    //    certain methods that expect it to be valid UTF-8 _may_ lead to
    //    undefined behaviour. Because of the previous item, this shouldn't be a
    //    problem.
    //
    // Long term we want to replace this function with a pure Inko
    // implementation, solving this problem entirely, but that proved to be too
    // much work at the time of writing this comment.
    let slice =
        str::from_utf8_unchecked(slice::from_raw_parts(bytes, size as _));

    let parsed = match slice {
        "Infinity" => Ok(f64::INFINITY),
        "-Infinity" => Ok(f64::NEG_INFINITY),
        _ => slice.parse::<f64>(),
    };

    parsed
        .map(|v| InkoResult::ok(v.to_bits() as _))
        .unwrap_or_else(|_| InkoResult::none())
}

#[no_mangle]
pub unsafe extern "system" fn inko_string_chars(
    string: *const InkoString,
) -> *mut u8 {
    let string = InkoString::read(string);

    // Safety: a Graphemes takes a reference to a slice of bytes. The standard
    // library implements a wrapper around this native type that holds on to the
    // string we're iterating over, preventing the slice from being invalidated
    // while this iterator still exists.
    //
    // Graphemes isn't FFI safe, so we have to work around this by casting it to
    // a regular raw pointer.
    Box::into_raw(Box::new(string.graphemes(true))) as _
}

#[no_mangle]
pub unsafe extern "system" fn inko_string_chars_next(
    state: *const State,
    iter: *mut u8,
) -> InkoResult {
    let iter = &mut *(iter as *mut Graphemes);

    iter.next()
        .map(|v| {
            let string =
                InkoString::alloc((*state).string_class, v.to_string());

            InkoResult::ok(string as _)
        })
        .unwrap_or_else(InkoResult::none)
}

#[no_mangle]
pub unsafe extern "system" fn inko_string_chars_drop(iter: *mut u8) {
    drop(Box::from_raw(iter as *mut Graphemes));
}

#[no_mangle]
pub unsafe extern "system" fn inko_string_slice_bytes(
    state: *const State,
    string: *const InkoString,
    start: i64,
    length: i64,
) -> *const ByteArray {
    let string = InkoString::read(string);
    let end = min((start + length) as usize, string.len());
    let bytes = if start < 0 || length <= 0 || start as usize >= end {
        Vec::new()
    } else {
        string.as_bytes()[start as usize..end].to_vec()
    };

    ByteArray::alloc((*state).byte_array_class, bytes)
}

#[no_mangle]
pub unsafe extern "system" fn inko_string_from_pointer(
    state: *const State,
    ptr: *const c_char,
) -> *const InkoString {
    let val = CStr::from_ptr(ptr).to_string_lossy().into_owned();

    InkoString::alloc((*state).string_class, val)
}
