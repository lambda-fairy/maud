extern crate alloc;

use alloc::string::String;

pub fn escape_to_string(input: &str, output: &mut String) {
    for b in input.bytes() {
        match b {
            b'&' => output.push_str("&amp;"),
            b'<' => output.push_str("&lt;"),
            b'>' => output.push_str("&gt;"),
            b'"' => output.push_str("&quot;"),
            _ => unsafe { output.as_mut_vec().push(b) },
        }
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
}
