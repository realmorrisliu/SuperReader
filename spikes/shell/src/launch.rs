use std::ffi::CStr;
use std::sync::Mutex;

use nsh_platform_macos::objc2::rc::Retained;
use nsh_platform_macos::objc2::runtime::{AnyClass, AnyObject, Imp, Sel};
use nsh_platform_macos::objc2::{ffi, msg_send};
use objc2_foundation::NSString;

use crate::books;

static OPEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn take_opens() -> Vec<String> {
    OPEN.lock()
        .map(|mut queued| std::mem::take(&mut *queued))
        .unwrap_or_default()
}

fn queue(path: String) {
    let Ok(mut queued) = OPEN.lock() else {
        return;
    };
    if !queued.iter().any(|have| have == &path) {
        queued.push(path);
    }
}

fn accept(raw: &str) {
    let path = raw.trim().trim_end_matches('/');
    if books::parse_book(path).is_some() {
        queue(path.to_string());
    }
}

fn ns_string(ptr: *mut AnyObject) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { (*(ptr.cast::<NSString>())).to_string() }
}

unsafe extern "C-unwind" fn open_urls(
    _this: *mut AnyObject,
    _cmd: Sel,
    _app: *mut AnyObject,
    urls: *mut AnyObject,
) {
    if urls.is_null() {
        return;
    }
    let urls = unsafe { &*urls };
    let count: usize = unsafe { msg_send![urls, count] };
    for index in 0..count {
        let url: Option<Retained<AnyObject>> = unsafe { msg_send![urls, objectAtIndex: index] };
        let Some(url) = url else { continue };
        let path: Option<Retained<NSString>> = unsafe { msg_send![&*url, path] };
        if let Some(path) = path {
            accept(&path.to_string());
        }
    }
}

unsafe extern "C-unwind" fn open_file(
    _this: *mut AnyObject,
    _cmd: Sel,
    _app: *mut AnyObject,
    filename: *mut AnyObject,
) -> bool {
    accept(&ns_string(filename));
    true
}

unsafe extern "C-unwind" fn open_files(
    _this: *mut AnyObject,
    _cmd: Sel,
    _app: *mut AnyObject,
    files: *mut AnyObject,
) {
    if files.is_null() {
        return;
    }
    let files = unsafe { &*files };
    let count: usize = unsafe { msg_send![files, count] };
    for index in 0..count {
        let file: Option<Retained<NSString>> = unsafe { msg_send![files, objectAtIndex: index] };
        if let Some(file) = file {
            accept(&file.to_string());
        }
    }
}

fn add(class: &AnyClass, name: &CStr, imp: Imp, types: &CStr) {
    let ptr = std::ptr::from_ref(class).cast_mut();
    let added = unsafe { ffi::class_addMethod(ptr, Sel::register(name), imp, types.as_ptr()) };
    if !added.as_bool() {
        eprintln!("document open hook {} was already present", name.to_string_lossy());
    }
}

pub fn install_document_open() {
    let Some(class) = AnyClass::get(c"NshAppDelegate") else {
        eprintln!("NshAppDelegate is missing; documents will not open");
        return;
    };
    add(
        class,
        c"application:openURLs:",
        unsafe { std::mem::transmute::<unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject, *mut AnyObject), Imp>(open_urls) },
        c"v@:@@",
    );
    add(
        class,
        c"application:openFile:",
        unsafe { std::mem::transmute::<unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject, *mut AnyObject) -> bool, Imp>(open_file) },
        c"B@:@@",
    );
    add(
        class,
        c"application:openFiles:",
        unsafe { std::mem::transmute::<unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject, *mut AnyObject), Imp>(open_files) },
        c"v@:@@",
    );
}
