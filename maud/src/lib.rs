//! Internal HTML utilities

#![experimental = "These functions should not be called directly.
Use the macros in `maud_macros` instead."]

use std::io::IoResult;

// http://www.w3.org/html/wg/drafts/html/master/single-page.html#escapingString

#[inline]
pub fn escape_attribute(c: char, w: &mut Writer) -> IoResult<()> {
    match c {
        '&' => w.write_str("&amp;"),
        '\u{A0}' => w.write_str("&nbsp;"),
        '"' => w.write_str("&quot;"),
        _ => w.write_char(c),
    }
}

#[inline]
pub fn escape_non_attribute(c: char, w: &mut Writer) -> IoResult<()> {
    match c {
        '&' => w.write_str("&amp;"),
        '\u{A0}' => w.write_str("&nbsp;"),
        '<' => w.write_str("&lt;"),
        '>' => w.write_str("&gt;"),
        _ => w.write_char(c),
    }
}
