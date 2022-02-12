use crate::html_writer::HtmlWriter;
use crate::schema::Schema;
use crate::tag::Tag;
use std::io;

pub struct Mapper<S, W> {
    schema: S,
    writer: W,
}

impl<S: Schema, W: HtmlWriter> Mapper<S, W> {
    pub fn new(s: S, w: W) -> Self {
        Self {
            schema: s,
            writer: w,
        }
    }

    pub fn write_header_end(&mut self, level: u8) -> io::Result<usize> {
        self.writer
            .write(Tag::header_end(&self.schema, level).as_bytes())?;
        self.writer.write_ln()
    }

    pub fn write_br(&mut self) -> io::Result<usize> {
        self.writer.write_br()?;
        self.writer.write_ln()
    }

    pub fn write_html_space(&mut self) -> io::Result<usize> {
        self.writer.write_html_space()
    }

    pub fn write_html_tab(&mut self) -> io::Result<usize> {
        self.writer.write_html_tab()
    }

    pub fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.writer.write(data)
    }

    pub fn write_tag_start(&mut self, tag: Tag) -> io::Result<usize> {
        self.writer.write(tag.start_tag(&self.schema).as_bytes())?;
        self.writer.write_ln()
    }

    pub fn write_tag_end(&mut self, tag: Tag) -> io::Result<()> {
        self.writer.write(tag.end_tag(&self.schema).as_bytes())?;
		W::set_used_tag(tag);
		
		Ok(())
    }

	pub fn write_css(&mut self) -> io::Result<()> {
		self.writer.write_css(&self.schema)
	}
}
