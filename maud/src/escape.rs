// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
// !!!!! PLEASE KEEP THIS IN SYNC WITH `maud_macros/src/escape.rs` !!!!!
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

extern crate alloc;

use alloc::string::String;

pub fn escape_to_string(input: &str, output: &mut String) {
    // Scan for the next character that needs escaping and copy the run of
    // safe bytes before it in a single `push_str`. This is much faster than
    // pushing one byte at a time for the common case of mostly-safe text.
    //
    // All four escaped characters are single-byte ASCII (< 0x80), so they
    // never occur inside a multi-byte UTF-8 sequence; slicing `input` at their
    // indices always lands on a character boundary.
    let bytes = input.as_bytes();
    let mut last = 0;
    for (i, &b) in bytes.iter().enumerate() {
        let replacement = match b {
            b'&' => "&amp;",
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'"' => "&quot;",
            _ => continue,
        };
        if last < i {
            output.push_str(&input[last..i]);
        }
        output.push_str(replacement);
        last = i + 1;
    }
    if last < bytes.len() {
        output.push_str(&input[last..]);
    }
}

#[cfg(test)]
mod test {
    extern crate alloc;

    use super::escape_to_string;
    use alloc::string::String;

    #[test]
    fn it_works() {
        let mut s = String::new();
        escape_to_string("<script>launchMissiles()</script>", &mut s);
        assert_eq!(s, "&lt;script&gt;launchMissiles()&lt;/script&gt;");
    }

    #[test]
    fn no_special_characters() {
        let mut s = String::new();
        escape_to_string("hello world", &mut s);
        assert_eq!(s, "hello world");
    }

    #[test]
    fn empty_string() {
        let mut s = String::new();
        escape_to_string("", &mut s);
        assert_eq!(s, "");
    }

    #[test]
    fn all_special_characters() {
        let mut s = String::new();
        escape_to_string("&<>\"", &mut s);
        assert_eq!(s, "&amp;&lt;&gt;&quot;");
    }

    #[test]
    fn adjacent_and_boundary_specials() {
        let mut s = String::new();
        escape_to_string("<<a&&b>>", &mut s);
        assert_eq!(s, "&lt;&lt;a&amp;&amp;b&gt;&gt;");
    }

    #[test]
    fn preserves_multibyte_utf8() {
        let mut s = String::new();
        escape_to_string("café <naïve> ☃ & 日本語", &mut s);
        assert_eq!(s, "café &lt;naïve&gt; ☃ &amp; 日本語");
    }

    #[test]
    fn appends_to_existing_buffer() {
        let mut s = String::from("prefix:");
        escape_to_string("a<b", &mut s);
        assert_eq!(s, "prefix:a&lt;b");
    }
}
