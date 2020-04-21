use super::Error;
use std::cell::RefCell;
use std::convert::Into;
use std::io::prelude::*;
use std::rc::Rc;

pub struct PrettyWriter<W: Write> {
    writer: Rc<RefCell<W>>,
    indent: u32,
    indent_bytes: &'static str,
    continuation_bytes: &'static str,
}

impl<W: Write> Clone for PrettyWriter<W> {
    fn clone(&self) -> Self {
        PrettyWriter {
            writer: self.writer.clone(),
            indent: self.indent,
            indent_bytes: self.indent_bytes,
            continuation_bytes: DEFAULT_CONTINUATION_BYTES,
        }
    }
}

const DEFAULT_CONTINUATION_BYTES: &str = "    ";

impl<W: Write> PrettyWriter<W> {
    /// Create a new `PrettyWriter` with `indent` initial units of indentation
    pub fn new_with_indent(writer: W, indent: u32, indent_bytes: &'static str) -> Self {
        PrettyWriter {
            writer: Rc::new(RefCell::new(writer)),
            indent,
            indent_bytes,
            continuation_bytes: DEFAULT_CONTINUATION_BYTES,
        }
    }

    /// Create a new `PrettyWriter` with no initial indentation
    pub fn new(writer: W, indent_bytes: &'static str) -> Self {
        PrettyWriter::new_with_indent(writer, 0, indent_bytes)
    }

    /// Create a writer based on a existing writer, but with no indentation`
    #[allow(dead_code)]
    pub fn new_from_writer(&mut self) -> Self {
        PrettyWriter {
            writer: self.writer.clone(),
            indent: 0,
            indent_bytes: self.indent_bytes.clone(),
            continuation_bytes: DEFAULT_CONTINUATION_BYTES,
        }
    }

    /// Create an indented block within the current `PrettyWriter`
    pub fn new_block(&mut self) -> Self {
        PrettyWriter {
            writer: self.writer.clone(),
            indent: self.indent + 1,
            indent_bytes: self.indent_bytes.clone(),
            continuation_bytes: DEFAULT_CONTINUATION_BYTES,
        }
    }

    fn _write_all<T: AsRef<[u8]>>(writer: &mut W, buf: T) -> Result<(), Error> {
        let buf = buf.as_ref();
        writer.write_all(buf).map_err(Into::into)
    }

    /// Return the current indentation level
    #[allow(dead_code)]
    pub fn indent_level(&self) -> u32 {
        self.indent
    }

    /// Output an indentation string
    pub fn indent(&mut self) -> Result<&mut Self, Error> {
        let indent_bytes = &self.indent_bytes.clone();
        {
            let mut writer = self.writer.borrow_mut();
            for _ in 0..self.indent {
                Self::_write_all(&mut writer, indent_bytes)?
            }
        }
        Ok(self)
    }

    /// Output a space
    #[allow(dead_code)]
    pub fn space(&mut self) -> Result<&mut Self, Error> {
        Self::_write_all(&mut self.writer.borrow_mut(), b" ")?;
        Ok(self)
    }

    /// Output an end of line
    pub fn eol(&mut self) -> Result<&mut Self, Error> {
        Self::_write_all(&mut self.writer.borrow_mut(), b"\n")?;
        Ok(self)
    }

    /// Output a block separator
    pub fn eob(&mut self) -> Result<&mut Self, Error> {
        self.eol()
    }

    /// Continuation
    pub fn continuation(&mut self) -> Result<&mut Self, Error> {
        self.indent()?;
        let continuation_bytes = &self.continuation_bytes.clone();
        Self::_write_all(&mut self.writer.borrow_mut(), continuation_bytes)?;
        Ok(self)
    }

    /// Write raw data
    pub fn write<T: AsRef<[u8]>>(&mut self, buf: T) -> Result<&mut Self, Error> {
        let buf = buf.as_ref();
        Self::_write_all(&mut self.writer.borrow_mut(), buf)?;
        Ok(self)
    }

    /// Indent, write raw data and terminate with an end of line
    pub fn write_line<T: AsRef<[u8]>>(&mut self, buf: T) -> Result<&mut Self, Error> {
        let buf = buf.as_ref();
        self.indent()?.write(buf)?.eol()
    }
}
