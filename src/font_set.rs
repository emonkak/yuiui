use fontconfig::fontconfig as fc;
use std::cmp;
use std::ffi::CString;
use std::mem;
use std::os::raw::*;
use std::ptr;
use std::str::CharIndices;
use x11::xft;
use x11::xlib;
use x11::xrender;

static FC_CHARSET: &'static [u8] = b"charset\0";
static FC_FAMILY: &'static [u8] = b"family\0";
static FC_PIXEL_SIZE: &'static [u8] = b"pixelsize\0";

pub struct FontSet {
    fontset: *mut fc::FcFontSet,
    pattern: *mut fc::FcPattern,
    charsets: Vec<*mut fc::FcCharSet>,
}

impl FontSet {
    pub fn new(font_family: &str) -> Option<FontSet> {
        unsafe {
            let font_family = CString::new(font_family).ok()?;

            let pattern = fc::FcPatternCreate();
            fc::FcPatternAddString(
                pattern,
                FC_FAMILY.as_ptr() as *mut c_char,
                font_family.as_ptr() as *const u8
            );
            fc::FcConfigSubstitute(
                ptr::null_mut(),
                pattern,
                fc::FcMatchPattern
            );
            fc::FcDefaultSubstitute(pattern);

            let mut result: fc::FcResult = fc::FcResultNoMatch;
            let fontset = fc::FcFontSort(
                ptr::null_mut(),
                pattern,
                1,
                ptr::null_mut(),
                &mut result
            );

            if result != fc::FcResultMatch || (*fontset).nfont == 0 {
                return None;
            }

            Some(FontSet {
                charsets: Vec::new(),
                fontset,
                pattern,
            })
        }
    }

    pub fn render_line_text<'a>(
        &mut self,
        display: *mut xlib::Display,
        draw: *mut xft::XftDraw,
        color: *mut xft::XftColor,
        font_size: f64,
        x: i32,
        y: i32,
        text: &'a str
    ) {
        let mut x_position = 0;
        let mut opened_fonts: Vec<*mut xft::XftFont> = Vec::new();
        let pattern = self.pattern;

        for chunk in ChunkIter::new(text, self.fontset, &mut self.charsets) {
            unsafe {
                if let Some(font) = opened_fonts
                    .iter()
                    .find(|&&font| (*font).pattern == chunk.font.cast())
                    .copied()
                    .or_else(|| chunk.open_font(display, pattern, font_size).map(|font| {
                        opened_fonts.push(font);
                        font
                    })) {
                    let mut extents: xrender::XGlyphInfo = mem::MaybeUninit::uninit().assume_init();
                    xft::XftTextExtentsUtf8(
                        display,
                        font,
                        chunk.text.as_ptr(),
                        chunk.text.len() as i32,
                        &mut extents
                    );

                    let ascent = (*font).ascent;
                    let height = cmp::max(font_size as i32, ascent);
                    let y_adjustment = (height - ascent) / 2;

                    xft::XftDrawStringUtf8(
                        draw,
                        color,
                        font,
                        x + x_position + (extents.x as i32),
                        y + y_adjustment + (extents.height as i32),
                        chunk.text.as_ptr(),
                        chunk.text.len() as i32
                    );

                    x_position += extents.width as i32;
                }
            }
        }

        for font in opened_fonts {
            unsafe {
                xft::XftFontClose(display, font);
            }
        }
    }
}

impl Drop for FontSet {
    fn drop(&mut self) {
        unsafe {
            for charset in self.charsets.iter() {
                fc::FcCharSetDestroy(*charset);
            }
            fc::FcFontSetDestroy(self.fontset);
            fc::FcPatternDestroy(self.pattern);
        }
    }
}

struct Chunk<'a> {
    text: &'a str,
    font: *mut fc::FcPattern,
}

impl<'a> Chunk<'a> {
    fn open_font(&self, display: *mut xlib::Display, pattern: *mut fc::FcPattern, font_size: f64) -> Option<*mut xft::XftFont> {
        unsafe {
            let pattern = fc::FcFontRenderPrepare(
                ptr::null_mut(),
                pattern,
                self.font
            );

            fc::FcPatternDel(pattern, FC_PIXEL_SIZE.as_ptr() as *mut c_char);
            fc::FcPatternAddDouble(
                pattern,
                FC_PIXEL_SIZE.as_ptr() as *mut c_char,
                font_size
            );

            let font = xft::XftFontOpenPattern(display, pattern.cast());
            if font.is_null() {
                fc::FcPatternDestroy(pattern);
                return None;
            }

            Some(font)
        }
    }
}

struct ChunkIter<'a> {
    current_font: Option<*mut fc::FcPattern>,
    current_index: usize,
    charsets: &'a mut Vec<*mut fc::FcCharSet>,
    fontset: *mut fc::FcFontSet,
    inner: CharIndices<'a>,
    text: &'a str,
}

impl<'a> ChunkIter<'a> {
    fn new(text: &'a str, fontset: *mut fc::FcFontSet, charsets: &'a mut Vec<*mut fc::FcCharSet>) -> Self {
        unsafe {
            assert!((*fontset).nfont > 0);
        }
        Self {
            charsets,
            current_font: None,
            current_index: 0,
            fontset,
            inner: text.char_indices(),
            text,
        }
    }

    #[inline]
    fn default_font(&self) -> *mut fc::FcPattern {
        unsafe {
            *(*self.fontset).fonts.offset(0)
        }
    }

    fn match_font(&mut self, c: char) -> Option<*mut fc::FcPattern> {
        unsafe {
            for i in 0..(*self.fontset).nfont {
                let font = *(*self.fontset).fonts.offset(i as isize);
                let charset = match self.charsets.get(i as usize) {
                    Some(charset) => *charset,
                    None => {
                        let mut charset: *mut fc::FcCharSet = ptr::null_mut();
                        let result = fc::FcPatternGetCharSet(
                            font,
                            FC_CHARSET.as_ptr() as *mut c_char,
                            0,
                            &mut charset
                        );
                        self.charsets.push(charset);
                        if result != fc::FcResultMatch {
                            continue;
                        }
                        charset
                    }
                };

                if !charset.is_null() && fc::FcCharSetHasChar(charset, c as u32) != 0 {
                    return Some(font);
                }
            }
        }

        None
    }
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, c)) = self.inner.next() {
            let matched_font = self.match_font(c);
            if i == 0 {
                self.current_font = matched_font;
            } else if self.current_font != matched_font {
                let result = Some(Chunk {
                    text: &self.text[self.current_index..i],
                    font: self.current_font.unwrap_or(self.default_font()),
                });
                self.current_font = matched_font;
                self.current_index = i;
                return result;
            }
        }

        if self.current_index < self.text.len() {
            let result = Some(Chunk {
                text: &self.text[self.current_index..],
                font: self.current_font.unwrap_or(self.default_font()),
            });
            self.current_font = None;
            self.current_index = self.text.len();
            return result;
        }

        None
    }
}
