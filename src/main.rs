// extern crate x11;
// extern crate fontconfig;
extern crate keytray;

use std::env;

use keytray::app;
use keytray::config::Config;
use keytray::context::Context;

// use x11::xft;
// use fontconfig::fontconfig as fc;
// use std::ffi::CString;
// use std::mem;
// use std::os::raw::*;
// use std::ptr;

// static FC_FAMILY: &'static [u8] = b"family\0";
// static FC_PIXEL_SIZE: &'static [u8] = b"pixelsize\0";

fn main() {
    // unsafe {
    //     let pattern = fc::FcPatternCreate();
    //     let query = CString::new("Sans").unwrap();
    //
    //     fc::FcPatternAddString(
    //         pattern,
    //         FC_FAMILY.as_ptr() as *mut c_char,
    //         query.as_ptr() as *const u8
    //     );
    //     fc::FcPatternAddDouble(
    //         pattern,
    //         FC_PIXEL_SIZE.as_ptr() as *mut c_char,
    //         64.0
    //     );
    //     fc::FcConfigSubstitute(
    //         ptr::null_mut::<c_void>(),
    //         pattern,
    //         fc::FcMatchPattern
    //     );
    //     fc::FcDefaultSubstitute(pattern);
    //
    //     let mut result: fc::FcResult = mem::MaybeUninit::uninit().assume_init();
    //     let fontset = fc::FcFontSort(
    //         ptr::null_mut::<c_void>(),
    //         pattern,
    //         1,
    //         ptr::null_mut::<*mut c_void>(),
    //         &mut result
    //     );
    //
    //     for i in 0..(*fontset).nfont {
    //         let font = fc::FcFontRenderPrepare(
    //             ptr::null_mut::<c_void>(),
    //             pattern,
    //             *(*fontset).fonts.offset(i as isize)
    //         );
    //
    //         let mut family_ptr: *mut fc::FcChar8 = ptr::null_mut();
    //
    //         fc::FcPatternGetString(
    //             font,
    //             FC_FAMILY.as_ptr() as *mut c_char,
    //             0,
    //             &mut family_ptr
    //         );
    //
    //         let family = CString::from_raw(family_ptr as *mut i8);
    //
    //         println!("{}: {}", i, family.into_string().unwrap());
    //     }
    // }

    let args = env::args().collect();
    let config = Config::parse(args);
    let context = Context::new(config).unwrap();

    app::run(context);
}
