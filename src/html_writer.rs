use crate::schema::Schema;
use crate::tag;
use crate::tag::Tag;
use std::io;
use std::io::BufWriter;
use std::io::Write;

const TAG_LEN: usize = tag::LEN - 1 + 5 + 1;
static mut USED_TAG: [bool; TAG_LEN] = [false; TAG_LEN];

pub trait HtmlWriter {
    fn write(&mut self, data: &[u8]) -> io::Result<usize>;

    fn write_html_space(&mut self) -> io::Result<usize> {
        self.write(b"&nbsp;")
    }

    fn write_html_tab(&mut self) -> io::Result<usize> {
        self.write(b"&nbsp;&nbsp;&nbsp;&nbsp;")
    }

    fn write_br(&mut self) -> io::Result<usize> {
        self.write(b"<br>")
    }

    fn write_ln(&mut self) -> io::Result<usize> {
        self.write(b"\n")
    }

    fn set_used_tag(tag: Tag) {
        unsafe {
            *USED_TAG.get_unchecked_mut(tag.tag_index()) = true;
        }
    }

    fn write_css<S: Schema>(&mut self, s: &S) -> io::Result<()> {
        let res = self.write_css0(s);
        Self::clear_used_tag();
        res
    }

    fn write_css0<S: Schema>(&mut self, s: &S) -> io::Result<()> {
        self.write_ln()?;
        self.write(s.css_tag_start().as_bytes())?;
        self.write_ln()?;

        unsafe {
            for (idx, &used) in USED_TAG.iter().enumerate() {
                if used {
                    self.write(
                        match idx {
                            0 => s.h1_css(),
                            1 => s.h2_css(),
                            2 => s.h3_css(),
                            3 => s.h4_css(),
                            4 => s.h5_css(),
                            5 => s.h6_css(),
                            6 => s.block_quote_css(),
                            7 => s.task_list_done_css(),
                            8 => s.task_list_todo_css(),
                            _ => unreachable!(),
                        }
                        .as_bytes(),
                    )?;
                    self.write_ln()?;
                }
            }
        }

        self.write(s.css_tag_end().as_bytes())?;

        Ok(())
    }

    fn clear_used_tag() {
        unsafe {
            USED_TAG = [false; TAG_LEN];
        }
    }
}

pub struct HtmlWriterImpl<W: Write> {
    buf_writer: BufWriter<W>,
}

impl<W: Write> HtmlWriterImpl<W> {
    pub fn new(w: W) -> Self {
        Self {
            buf_writer: BufWriter::new(w),
        }
    }
}

impl<W: Write> HtmlWriter for HtmlWriterImpl<W> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.buf_writer.write(data)
    }
}
