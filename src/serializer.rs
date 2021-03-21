use std::io::Write;
use crate::schema::{Background, Image};
use anyhow::Result;

pub struct GnomeXMLBackgroundSerializer<'a, T: Write> {
    writer: &'a mut T,
}

impl<'a, T> GnomeXMLBackgroundSerializer<'a, T> where T: Write {
    pub fn new(writer: &'a mut T) -> Self {
        Self {
            writer
        }
    }

    pub fn serialize(&mut self, background: &Background) -> Result<()> {
        // By definition we can only find one starttime
        let mut biter = background.images.iter();
        match biter.next() {
            Some(Image::StartTime { year, month, day, hour, minute, second }) => {
                self.writer.write(b"<background>\n")?;
                self.writer.write(b"\t<starttime>\n")?;
                write!(self.writer, "\t\t<year>{}</year>\n", year)?;
                write!(self.writer, "\t\t<month>{}</month>\n", month)?;
                write!(self.writer, "\t\t<day>{}</day>\n", day)?;
                write!(self.writer, "\t\t<hour>{}</hour>\n", hour)?;
                write!(self.writer, "\t\t<minute>{}</minute>\n", minute)?;
                write!(self.writer, "\t\t<second>{}</second>\n", second)?;
                self.writer.write(b"\t</starttime>\n")?;
            },
            _ => return Err(anyhow::Error::msg("First item not startime. This is required by convention.")),
        }

        for entry in biter {
            match entry {
                Image::Static { duration, file } => {
                    write!(self.writer, "\t<static>\n")?;
                    write!(self.writer, "\t\t<duration>{}</duration>\n", duration)?;
                    write!(self.writer, "\t\t<file>{}</file>\n", file)?;
                    write!(self.writer, "\t</static>\n")?;
                }
                Image::Transition { kind, duration, from, to } => {
                    write!(self.writer, "\t<transition type=\"{}\">\n", kind)?;
                    write!(self.writer, "\t\t<duration>{}</duration>\n", duration)?;
                    write!(self.writer, "\t\t<from>{}</from>\n", from)?;
                    write!(self.writer, "\t\t<to>{}</to>\n", to)?;
                    write!(self.writer, "\t</transition>\n")?;
                }
                Image::StartTime {..} => {
                    return Err(anyhow::Error::msg("Cannot define starttime multiple times."))
                }
            }
        }
        write!(self.writer, "</background>")?;
        self.writer.flush()?;
        Ok(())
    }
}