// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Replacement for ICU library bindings using native Rust and the `regex` crate.

use std::cmp::Ordering;
use std::mem::MaybeUninit;
use std::ops::Range;

use stdext::arena::{Arena, ArenaString};

use crate::buffer::TextBuffer;
use crate::apperr;

#[derive(Clone, Copy)]
pub struct Encoding {
    pub label: &'static str,
    pub canonical: &'static str,
}

pub struct Encodings {
    pub preferred: &'static [Encoding],
    pub all: &'static [Encoding],
}

static ENCODINGS: Encodings = Encodings {
    preferred: &[
        Encoding { label: "UTF-8", canonical: "UTF-8" },
        Encoding { label: "UTF-8 BOM", canonical: "UTF-8 BOM" },
    ],
    all: &[
        Encoding { label: "UTF-8", canonical: "UTF-8" },
        Encoding { label: "UTF-8 BOM", canonical: "UTF-8 BOM" },
    ],
};

/// Returns a list of encodings supported (UTF-8 only for this shim).
pub fn get_available_encodings() -> &'static Encodings {
    &ENCODINGS
}

pub fn apperr_format(f: &mut std::fmt::Formatter<'_>, code: u32) -> std::fmt::Result {
    write!(f, "ICU Error (Stub): {code:#08x}")
}

pub fn init() -> apperr::Result<()> {
    Ok(())
}

/// Converts between two encodings. 
/// Only supports UTF-8 to UTF-8 copy for now.
pub struct Converter<'pivot> {
    _marker: std::marker::PhantomData<&'pivot mut [MaybeUninit<u16>]>,
}

impl<'pivot> Converter<'pivot> {
    pub fn new(
        _pivot_buffer: &'pivot mut [MaybeUninit<u16>],
        source_encoding: &str,
        target_encoding: &str,
    ) -> apperr::Result<Self> {
        if (source_encoding == "UTF-8" || source_encoding == "UTF-8 BOM") &&
           (target_encoding == "UTF-8" || target_encoding == "UTF-8 BOM") {
            Ok(Self { _marker: std::marker::PhantomData })
        } else {
            // ICU U_UNSUPPORTED_ERROR = 16
            Err(apperr::Error::new_icu(16))
        }
    }

    pub fn convert(
        &mut self,
        input: &[u8],
        output: &mut [MaybeUninit<u8>],
    ) -> apperr::Result<(usize, usize)> {
        let len = input.len().min(output.len());
        unsafe {
            std::ptr::copy_nonoverlapping(input.as_ptr(), output.as_mut_ptr() as *mut u8, len);
        }
        Ok((len, len))
    }
}

/// Compares two UTF-8 strings.
pub fn compare_strings(a: &[u8], b: &[u8]) -> Ordering {
    a.cmp(b)
}

/// Converts the given UTF-8 string to lower case.
pub fn fold_case<'a>(arena: &'a Arena, input: &str) -> ArenaString<'a> {
    let folded = input.to_lowercase();
    ArenaString::from_str(arena, &folded)
}

// -----------------------------------------------------------------------------------------
// Regex and Text implementation
// -----------------------------------------------------------------------------------------

/// A wrapper around the text content.
pub struct Text {
    pub content: String,
    tb_ptr: *const TextBuffer,
}

impl Drop for Text {
    fn drop(&mut self) {}
}

impl Text {
    /// Constructs a copy of the TextBuffer content into a String.
    /// Stores the TextBuffer pointer for later refresh.
    pub unsafe fn new(tb: &TextBuffer) -> apperr::Result<Self> {
        let mut t = Self { 
            content: String::new(), 
            tb_ptr: tb as *const _ 
        };
        t.refresh();
        Ok(t)
    }

    pub unsafe fn refresh(&mut self) {
        let tb = &*self.tb_ptr;
        self.content.clear();
        self.content.reserve(tb.text_length());
        
        let mut offset = 0;
        loop {
            let chunk = tb.read_forward(offset);
            if chunk.is_empty() {
                break;
            }
            self.content.push_str(&String::from_utf8_lossy(chunk));
            offset += chunk.len();
        }
    }
}

pub struct Regex {
    inner: regex::Regex,
    text: String,
    last_idx: usize,
    captures: Option<Vec<Range<usize>>>,
}

impl Regex {
    pub const CASE_INSENSITIVE: i32 = 1;
    pub const MULTILINE: i32 = 2;
    pub const LITERAL: i32 = 4;

    pub unsafe fn new(pattern: &str, flags: i32, text: &Text) -> apperr::Result<Self> {
        let pattern_string;
        let final_pattern = if (flags & Self::LITERAL) != 0 {
            pattern_string = regex::escape(pattern);
            &pattern_string
        } else {
            pattern
        };

        let mut builder = regex::RegexBuilder::new(final_pattern);
        
        if (flags & Self::CASE_INSENSITIVE) != 0 {
            builder.case_insensitive(true);
        }
        if (flags & Self::MULTILINE) != 0 {
            builder.multi_line(true);
        }
        
        match builder.build() {
            Ok(inner) => Ok(Self {
                inner,
                text: text.content.clone(),
                last_idx: 0,
                captures: None,
            }),
            Err(_) => Err(apperr::Error::new_icu(1)), // U_ILLEGAL_ARGUMENT_ERROR
        }
    }

    /// Updates the text content from the TextBuffer and resets search.
    /// This is called when the editor buffer has changed.
    pub unsafe fn set_text(&mut self, text: &mut Text, offset: usize) {
        text.refresh();
        self.text = text.content.clone();
        self.reset(offset);
    }

    pub fn reset(&mut self, offset: usize) {
        self.last_idx = offset;
        self.captures = None;
    }

    pub fn group_count(&mut self) -> i32 {
        if let Some(caps) = &self.captures {
            (caps.len() as i32).saturating_sub(1)
        } else {
            0
        }
    }

    pub fn group(&mut self, group: i32) -> Option<Range<usize>> {
        if let Some(caps) = &self.captures {
            caps.get(group as usize).cloned()
        } else {
            None
        }
    }
}

impl Iterator for Regex {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_idx > self.text.len() {
            return None;
        }

        match self.inner.captures_at(&self.text, self.last_idx) {
            Some(caps) => {
                let m = caps.get(0).unwrap();
                let range = m.start()..m.end();
                
                let mut groups = Vec::new();
                for i in 0..caps.len() {
                    if let Some(g) = caps.get(i) {
                        groups.push(g.start()..g.end());
                    } else {
                        groups.push(0..0);
                    }
                }
                self.captures = Some(groups);
                
                if range.start == range.end {
                     self.last_idx = range.end + 1;
                } else {
                     self.last_idx = range.end;
                }

                Some(range)
            }
            None => None,
        }
    }
}