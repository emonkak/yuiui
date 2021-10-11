use std::hash::Hash;
use std::io;
use std::ops::Range;
use std::str::CharIndices;

use super::FontDescriptor;

pub trait FontLoader {
    type Bundle;

    type FontId: Copy + Eq + Hash;

    fn load_bundle(&mut self, descriptor: &FontDescriptor) -> Option<Self::Bundle>;

    fn get_primary_font(&self, bundle: &Self::Bundle) -> Self::FontId;

    fn match_optimal_font(&self, bundle: &Self::Bundle, c: char) -> Option<Self::FontId>;

    fn load_font(&self, id: Self::FontId) -> io::Result<Vec<u8>>;

    fn split_segments<'a>(
        &'a self,
        bundle: &'a Self::Bundle,
        content: &'a str,
    ) -> SplitSegments<'a, Self, Self::Bundle, Self::FontId>
    where
        Self: Sized
    {
        SplitSegments::new(self, bundle, content)
    }
}

pub struct SplitSegments<'a, Loader, Bundle, FontId> {
    loader: &'a Loader,
    bundle: &'a Bundle,
    content: &'a str,
    primary_font: FontId,
    current_font: FontId,
    cursor: usize,
    char_indices: CharIndices<'a>,
}

impl<'a, Loader: FontLoader> SplitSegments<'a, Loader, Loader::Bundle, Loader::FontId> {
    fn new(loader: &'a Loader, bundle: &'a Loader::Bundle, content: &'a str) -> Self {
        let primary_font = loader.get_primary_font(bundle);
        Self {
            loader,
            bundle,
            content,
            primary_font,
            current_font: primary_font,
            cursor: 0,
            char_indices: content.char_indices(),
        }
    }
}

impl<'a, Loader: FontLoader> Iterator
    for SplitSegments<'a, Loader, Loader::Bundle, Loader::FontId>
{
    type Item = (Loader::FontId, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, c)) = self.char_indices.next() {
            let font = self
                .loader
                .match_optimal_font(&self.bundle, c)
                .unwrap_or(self.primary_font);
            if font != self.current_font {
                if i > 0 {
                    let result = (self.current_font, self.cursor..i);
                    self.cursor = i;
                    self.current_font = font;
                    return Some(result);
                } else {
                    self.cursor = i;
                    self.current_font = font;
                }
            }
        }

        if self.cursor < self.content.len() {
            let result = (self.current_font, self.cursor..self.content.len());
            self.cursor = self.content.len();
            return Some(result);
        }

        None
    }
}
