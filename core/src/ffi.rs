use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::json_types::Document;
use crate::parser::MarkdownParser;

/// Parse a markdown string and return a JSON representation.
///
/// # Safety
/// - `text` must be a valid, null-terminated UTF-8 C string.
/// - The returned pointer is heap-allocated and must be freed with `markdown_free_string`.
/// - Returns null on serialization failure.
#[no_mangle]
pub unsafe extern "C" fn markdown_parse(text: *const c_char) -> *mut c_char {
    if text.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(text) };
    let input = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let blocks = MarkdownParser::parse(input);
    let document = Document::from_blocks(blocks);

    match serde_json::to_string(&document) {
        Ok(json) => match CString::new(json) {
            Ok(c) => c.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Load a file from disk and return its contents as a C string.
///
/// # Safety
/// - `path` must be a valid, null-terminated UTF-8 C string.
/// - The returned pointer must be freed with `markdown_free_string`.
/// - Returns null on any IO or encoding error.
#[no_mangle]
pub unsafe extern "C" fn markdown_load_file(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match crate::file_loader::FileLoader::load_file(std::path::Path::new(path_str)) {
        Ok(content) => match CString::new(content) {
            Ok(c) => c.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Check whether a file path has a markdown extension.
///
/// # Safety
/// - `path` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn markdown_is_markdown_file(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    crate::file_loader::FileLoader::is_markdown_file(std::path::Path::new(path_str))
}

/// Free a string previously returned by any `markdown_*` function.
///
/// # Safety
/// - `s` must be a pointer previously returned by this library, or null.
/// - Do not call twice on the same pointer.
#[no_mangle]
pub unsafe extern "C" fn markdown_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(unsafe { CString::from_raw(s) });
    }
}
