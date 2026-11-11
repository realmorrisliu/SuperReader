use std::ffi::CStr;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::Mutex;

use nsh_platform_macos::objc2::runtime::{AnyClass, AnyObject, Imp, Sel};
use nsh_platform_macos::objc2::{ffi, msg_send};
use objc2_foundation::NSString;

use crate::books;

static OPEN: Mutex<Vec<String>> = Mutex::new(Vec::new());
static INSTALLED: AtomicBool = AtomicBool::new(false);
static HOOKED: AtomicBool = AtomicBool::new(false);
static ORIGINAL_SET_DELEGATE: AtomicPtr<()> = AtomicPtr::new(std::ptr::null_mut());

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
        trace(&format!("queued {path}"));
        queue(path.to_string());
    } else if !path.is_empty() {
        trace(&format!("rejected {path}"));
    }
}

fn trace(line: &str) {
    let dir = "/tmp/superreader-spike";
    let _ = std::fs::create_dir_all(dir);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{dir}/opens.log"))
    {
        let _ = writeln!(file, "{line}");
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
    let count: usize = unsafe { msg_send![urls, count] };
    for index in 0..count {
        let url: *mut AnyObject = unsafe { msg_send![urls, objectAtIndex: index] };
        if url.is_null() {
            continue;
        }
        let path: *mut NSString = unsafe { msg_send![url, path] };
        if path.is_null() {
            continue;
        }
        accept(&unsafe { &*path }.to_string());
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
        let file: *mut NSString = unsafe { msg_send![files, objectAtIndex: index] };
        if file.is_null() {
            continue;
        }
        accept(&unsafe { &*file }.to_string());
    }
}

fn add(class: &AnyClass, name: &CStr, imp: Imp, types: &CStr) {
    let ptr = std::ptr::from_ref(class).cast_mut();
    let sel = Sel::register(name);
    // 0.5.0 already implements application:openURLs: and turns the URL into
    // a UrlScheme event. class_addMethod cannot replace that, so the path
    // queue would never see a Finder open. class_replaceMethod adds the
    // method when it is missing and replaces it when it is already there.
    let previous = unsafe { ffi::class_replaceMethod(ptr, sel, imp, types.as_ptr()) };
    if previous.is_some() {
        trace(&format!("replaced {}", name.to_string_lossy()));
    }
}

pub fn install_document_open() {
    hook_set_delegate();
    install_methods();
}

fn hook_set_delegate() {
    if HOOKED.swap(true, Ordering::Relaxed) {
        return;
    }
    let Some(class) = AnyClass::get(c"NSApplication") else {
        HOOKED.store(false, Ordering::Relaxed);
        trace("NSApplication is missing");
        return;
    };
    let sel = Sel::register(c"setDelegate:");
    let method = unsafe { ffi::class_getInstanceMethod(std::ptr::from_ref(class), sel) };
    if method.is_null() {
        HOOKED.store(false, Ordering::Relaxed);
        trace("NSApplication setDelegate: is missing");
        return;
    }
    let replacement = unsafe {
        std::mem::transmute::<unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject), Imp>(
            set_delegate,
        )
    };
    let Some(previous) = (unsafe { ffi::method_setImplementation(method, replacement) }) else {
        HOOKED.store(false, Ordering::Relaxed);
        trace("could not replace setDelegate:");
        return;
    };
    ORIGINAL_SET_DELEGATE.store(
        unsafe { std::mem::transmute::<Imp, *mut ()>(previous) },
        Ordering::Release,
    );
}

unsafe extern "C-unwind" fn set_delegate(this: *mut AnyObject, cmd: Sel, delegate: *mut AnyObject) {
    let original = ORIGINAL_SET_DELEGATE.load(Ordering::Acquire);
    if !original.is_null() {
        let original: unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject) =
            unsafe { std::mem::transmute(original) };
        original(this, cmd, delegate);
    }
    // NshAppDelegate is registered when run_app allocates it, which is the
    // line before setDelegate:. The open methods have to be on the class
    // before the run loop delivers the launch document.
    install_methods();
}

fn install_methods() {
    if INSTALLED.load(Ordering::Relaxed) {
        return;
    }
    let Some(class) = AnyClass::get(c"NshAppDelegate") else {
        return;
    };
    add(
        class,
        c"application:openURLs:",
        unsafe {
            std::mem::transmute::<
                unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject, *mut AnyObject),
                Imp,
            >(open_urls)
        },
        c"v@:@@",
    );
    add(
        class,
        c"application:openFile:",
        unsafe {
            std::mem::transmute::<
                unsafe extern "C-unwind" fn(
                    *mut AnyObject,
                    Sel,
                    *mut AnyObject,
                    *mut AnyObject,
                ) -> bool,
                Imp,
            >(open_file)
        },
        c"B@:@@",
    );
    add(
        class,
        c"application:openFiles:",
        unsafe {
            std::mem::transmute::<
                unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject, *mut AnyObject),
                Imp,
            >(open_files)
        },
        c"v@:@@",
    );
    INSTALLED.store(true, Ordering::Relaxed);
    trace("document open hook installed");
}
