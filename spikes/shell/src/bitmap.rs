use std::ffi::c_void;

use objc2_core_foundation::{CFRetained, CGFloat};
use objc2_core_graphics::{
    CGBitmapInfo, CGColorRenderingIntent, CGColorSpace, CGDataProvider, CGImage, CGImageAlphaInfo,
};

unsafe extern "C-unwind" fn release_bytes(
    info: *mut c_void,
    _data: std::ptr::NonNull<c_void>,
    _size: usize,
) {
    if !info.is_null() {
        drop(Box::from_raw(info as *mut Box<[u8]>));
    }
}

pub fn flip_rows(width: u32, bytes: &[u8]) -> Vec<u8> {
    let stride = width as usize * 4;
    if stride == 0 || bytes.len() % stride != 0 {
        return bytes.to_vec();
    }
    let rows = bytes.len() / stride;
    let mut out = vec![0u8; bytes.len()];
    for row in 0..rows {
        let src = (rows - 1 - row) * stride;
        let dst = row * stride;
        out[dst..dst + stride].copy_from_slice(&bytes[src..src + stride]);
    }
    out
}

pub fn rgba_image(
    width: u32,
    height: u32,
    bytes: Vec<u8>,
    premultiplied: bool,
) -> Option<CFRetained<CGImage>> {
    let expected = width as usize * height as usize * 4;
    if bytes.len() != expected || width == 0 || height == 0 {
        return None;
    }
    let boxed: Box<[u8]> = bytes.into_boxed_slice();
    let ptr = boxed.as_ptr();
    let len = boxed.len();
    let info = Box::into_raw(Box::new(boxed));
    let alpha = if premultiplied {
        CGImageAlphaInfo::PremultipliedLast
    } else {
        CGImageAlphaInfo::Last
    };
    let bitmap = CGBitmapInfo(alpha.0 | CGBitmapInfo::ByteOrder32Big.0);
    unsafe {
        let provider = CGDataProvider::with_data(
            info as *mut c_void,
            ptr as *const c_void,
            len,
            Some(release_bytes),
        )?;
        let space = CGColorSpace::new_device_rgb()?;
        CGImage::new(
            width as usize,
            height as usize,
            8,
            32,
            (width * 4) as usize,
            Some(&space),
            bitmap,
            Some(&provider),
            std::ptr::null::<CGFloat>(),
            false,
            CGColorRenderingIntent::RenderingIntentDefault,
        )
    }
}
