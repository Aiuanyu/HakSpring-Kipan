//! Lightweight WebAssembly entry point for `two-line`.
//!
//! Exports a minimal linear-memory ABI (zero `wasm-bindgen` dependency)
//! returning `(ptr << 32) | len` packed into `u64` for UTF-8 / JSON payloads.

use std::slice;

#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        drop(Vec::from_raw_parts(ptr, 0, len));
    }
}

fn pack_output(bytes: Vec<u8>) -> u64 {
    let len = bytes.len();
    let ptr = alloc(len);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
    }
    ((ptr as u64) << 32) | (len as u64)
}

unsafe fn read_utf8(ptr: *const u8, len: usize) -> String {
    if ptr.is_null() || len == 0 {
        return String::new();
    }
    let slice = slice::from_raw_parts(ptr, len);
    String::from_utf8_lossy(slice).into_owned()
}

#[no_mangle]
pub unsafe extern "C" fn convert_kipan_text_json(
    text_ptr: *const u8,
    text_len: usize,
    auto_space_romance: u8,
    order_ptr: *const u8,
    order_len: usize,
) -> u64 {
    let text = read_utf8(text_ptr, text_len);
    let force_order = read_utf8(order_ptr, order_len);
    let order = if force_order.is_empty() {
        "auto"
    } else {
        force_order.as_str()
    };
    let res = two_line::convert_kipan_text(&text, auto_space_romance != 0, order);
    let json = serde_json::to_vec(&res).unwrap_or_default();
    pack_output(json)
}

#[no_mangle]
pub unsafe extern "C" fn convert_punct_to_romance_utf8(ptr: *const u8, len: usize) -> u64 {
    let input = read_utf8(ptr, len);
    let out = two_line::convert_punct_to_romance(&input);
    pack_output(out.into_bytes())
}

#[no_mangle]
pub unsafe extern "C" fn convert_punct_to_han_utf8(ptr: *const u8, len: usize) -> u64 {
    let input = read_utf8(ptr, len);
    let out = two_line::convert_punct_to_han(&input);
    pack_output(out.into_bytes())
}
