use std::{fmt, io, str};

pub struct TextWriter<T>(pub T);

impl<T: fmt::Write> io::Write for TextWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s =
            str::from_utf8(buf).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        self.0
            .write_str(s)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
