#[allow(dead_code)]
mod ffi;

use std::ffi::{CStr, CString};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io;
use std::io::Read;
use std::os::raw::*;
use std::ptr;

use crate::text::{FontDescriptor, FontFamily, FontStretch, FontStyle};

static SERIF_FAMILY: &'static str = "Serif\0";
static SANS_SERIF_FAMILY: &'static str = "Sans\0";
static MONOSPACE_FAMILY: &'static str = "Monospace\0";

pub struct FontLoader;

pub struct FontBundle {
    pattern: *mut ffi::FcPattern,
    fontset: *mut ffi::FcFontSet,
    coverage: *mut ffi::FcCharSet,
    charsets: Vec<*mut ffi::FcCharSet>,
}

#[derive(Clone, Copy, Debug)]
pub struct FontId {
    pattern: *mut ffi::FcPattern,
}

impl crate::text::FontLoader for FontLoader {
    type Bundle = FontBundle;

    type FontId = FontId;

    fn load_bundle(&mut self, descriptor: &FontDescriptor) -> Option<Self::Bundle> {
        unsafe {
            let pattern = create_pattern(descriptor);

            ffi::FcConfigSubstitute(ptr::null_mut(), pattern, ffi::FcMatchKind::Pattern);
            ffi::FcDefaultSubstitute(pattern);

            let mut result = ffi::FcResult::NoMatch;
            let fontset =
                ffi::FcFontSort(ptr::null_mut(), pattern, 1, ptr::null_mut(), &mut result);

            if result != ffi::FcResult::Match || (*fontset).nfont == 0 {
                return None;
            }

            let mut coverage = ffi::FcCharSetNew();
            let mut charsets = Vec::with_capacity((*fontset).nfont as usize);

            for i in 0..(*fontset).nfont {
                let font = *(*fontset).fonts.offset(i as isize);

                let mut charset: *mut ffi::FcCharSet = ptr::null_mut();
                let result = ffi::FcPatternGetCharSet(
                    font,
                    ffi::FC_CHARSET.as_ptr() as *mut c_char,
                    0,
                    &mut charset,
                );

                if result == ffi::FcResult::Match {
                    coverage = ffi::FcCharSetUnion(coverage, charset);
                }

                charsets.push(charset);
            }

            Some(FontBundle {
                pattern,
                fontset,
                charsets,
                coverage,
            })
        }
    }

    fn get_primary_font(&self, bundle: &Self::Bundle) -> Self::FontId {
        unsafe {
            FontId {
                pattern: *(*bundle.fontset).fonts.offset(0),
            }
        }
    }

    fn match_optimal_font(&self, bundle: &Self::Bundle, c: char) -> Option<Self::FontId> {
        unsafe {
            if ffi::FcCharSetHasChar(bundle.coverage, c as _) == 0 {
                return None;
            }

            for i in 0..(*bundle.fontset).nfont {
                let charset = bundle.charsets[i as usize];
                if !charset.is_null() && ffi::FcCharSetHasChar(charset, c as _) != 0 {
                    let pattern = *(*bundle.fontset).fonts.offset(i as isize);
                    return Some(FontId { pattern });
                }
            }
        }

        None
    }

    fn load_font(&self, id: Self::FontId) -> io::Result<Vec<u8>> {
        let path = unsafe {
            let mut file_string_ptr = ptr::null_mut();
            let result = ffi::FcPatternGetString(
                id.pattern,
                ffi::FC_FILE.as_ptr() as *mut c_char,
                0,
                &mut file_string_ptr,
            );
            if result == ffi::FcResult::Match {
                Some(
                    CStr::from_ptr(file_string_ptr as *mut _)
                        .to_string_lossy()
                        .into_owned(),
                )
            } else {
                None
            }
        };

        if let Some(path) = path {
            let mut buffer = Vec::new();
            let mut reader = File::open(path)?;
            let _ = reader.read_to_end(&mut buffer)?;
            Ok(buffer)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Font path cannnot be resolved",
            ))
        }
    }
}

impl Drop for FontBundle {
    fn drop(&mut self) {
        unsafe {
            ffi::FcCharSetDestroy(self.coverage);
            ffi::FcFontSetDestroy(self.fontset);
            ffi::FcPatternDestroy(self.pattern);
        }
    }
}

impl PartialEq for FontId {
    fn eq(&self, other: &Self) -> bool {
        unsafe { ffi::FcPatternEqual(self.pattern, other.pattern) != 0 }
    }
}

impl Eq for FontId {}

impl Hash for FontId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let hash = unsafe { ffi::FcPatternHash(self.pattern) };
        state.write_u32(hash);
    }
}

unsafe fn create_pattern(descriptor: &FontDescriptor) -> *mut ffi::FcPattern {
    let pattern = ffi::FcPatternCreate();

    match &descriptor.family {
        FontFamily::Name(name) => {
            if let Ok(name_str) = CString::new(name.as_str()) {
                ffi::FcPatternAddString(
                    pattern,
                    ffi::FC_FAMILY.as_ptr() as *mut c_char,
                    name_str.as_ptr() as *mut c_uchar,
                );
            }
        }
        FontFamily::Serif => {
            ffi::FcPatternAddString(
                pattern,
                ffi::FC_FAMILY.as_ptr() as *mut c_char,
                SERIF_FAMILY.as_ptr(),
            );
        }
        FontFamily::SansSerif => {
            ffi::FcPatternAddString(
                pattern,
                ffi::FC_FAMILY.as_ptr() as *mut c_char,
                SANS_SERIF_FAMILY.as_ptr(),
            );
        }
        FontFamily::Monospace => {
            ffi::FcPatternAddString(
                pattern,
                ffi::FC_FAMILY.as_ptr() as *mut c_char,
                MONOSPACE_FAMILY.as_ptr(),
            );
        }
    };

    let weight = ffi::FcWeightFromOpenTypeDouble(descriptor.weight.0 as _);
    ffi::FcPatternAddDouble(pattern, ffi::FC_WEIGHT.as_ptr() as *mut c_char, weight);

    let slant = match descriptor.style {
        FontStyle::Italic => ffi::FC_SLANT_ITALIC,
        FontStyle::Normal => ffi::FC_SLANT_ROMAN,
        FontStyle::Oblique => ffi::FC_SLANT_OBLIQUE,
    };
    ffi::FcPatternAddInteger(pattern, ffi::FC_SLANT.as_ptr() as *mut c_char, slant);

    let width = match descriptor.stretch {
        FontStretch::UltraCondensed => ffi::FC_WIDTH_ULTRACONDENSED,
        FontStretch::ExtraCondensed => ffi::FC_WIDTH_EXTRACONDENSED,
        FontStretch::Condensed => ffi::FC_WIDTH_CONDENSED,
        FontStretch::SemiCondensed => ffi::FC_WIDTH_SEMICONDENSED,
        FontStretch::Normal => ffi::FC_WIDTH_NORMAL,
        FontStretch::SemiExpanded => ffi::FC_WIDTH_SEMIEXPANDED,
        FontStretch::Expanded => ffi::FC_WIDTH_EXPANDED,
        FontStretch::ExtraExpanded => ffi::FC_WIDTH_EXTRAEXPANDED,
        FontStretch::UltraExpanded => ffi::FC_WIDTH_ULTRAEXPANDED,
    };
    ffi::FcPatternAddInteger(pattern, ffi::FC_WIDTH.as_ptr() as *mut c_char, width);

    pattern
}
