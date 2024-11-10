use std::{io, str};

pub struct StringWriter<'a>(pub &'a mut String);

impl io::Write for StringWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        str::from_utf8(buf)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
            .map(|s| {
                self.0.push_str(s);
                buf.len()
            })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
