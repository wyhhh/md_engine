use crate::html_writer::HtmlWriter;
use crate::mapper::Mapper;
use crate::schema::Schema;
use crate::tag::Tag;
use crate::tokenizer::Token;
use crate::tokenizer::Tokenizer;
use crate::CowStr;
use std::fmt::Debug;
use std::io;
use std::io::Read;

pub trait Parser {
    fn parse_and_write(&mut self) -> Result<(), ParseError>;
}

#[derive(Debug)]
pub enum ParseError {
    SytaxError(SyntaxError),
    IoError(io::Error),
}

pub struct SyntaxError {
    msg: CowStr,
    line_num: u32,
    column_num: u32,
}

struct Record {
    line_num: u32,
    column_num: u32,
}

pub struct StatefulParser<'a, S, W, R: Read> {
    tokenizer: Tokenizer<'a, R>,
    mapper: Mapper<S, W>,
    /* CONTEXT: */
    state: State,
    last_tag: Tag,
    record: Record,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Start,
    // spaces, tabs, unbindinged text
    PureText,
    Ln,
    Tag(Tag),
    Value,
    BlockValueLn,
}

impl Record {
    fn new() -> Self {
        Self {
            line_num: 1,
            column_num: 0,
        }
    }

    fn update_nums(&mut self, token_char_len: u32) {
        if token_char_len == 0 {
            self.line_num += 1;
            self.column_num = 0;
        } else {
            self.column_num += token_char_len;
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(io_error: io::Error) -> Self {
        Self::IoError(io_error)
    }
}
impl SyntaxError {
    fn new<S: Into<CowStr>>(msg: S, line_num: u32, column_num: u32) -> Self {
        Self {
            msg: msg.into(),
            line_num,
            column_num,
        }
    }
}

impl Debug for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "[Syntax Error] at line {} column {}, message: \"{}\"",
            self.line_num, self.column_num, self.msg
        ))
    }
}

impl<'a, S: Schema, W: HtmlWriter, R: Read + 'a> StatefulParser<'a, S, W, R> {
    pub fn new(r: R, s: S, w: W) -> Self {
        Self {
            record: Record::new(),
            tokenizer: Tokenizer::new(r),
            state: State::Start,
            last_tag: Tag::None,
            mapper: Mapper::new(s, w),
        }
    }

    fn try_solve_header_end(&mut self) -> io::Result<bool> {
        if let Tag::Header(level) = self.last_tag {
            self.mapper.write_header_end(level)?;
            self.state = State::Ln;
            self.last_tag = Tag::None;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn solve_end_and_start(&mut self, tag: Tag) -> io::Result<()> {
        self.solve_end()?;
        self.solve_start(tag)
    }

    fn solve_start(&mut self, tag: Tag) -> io::Result<()> {
        self.mapper.write_tag_start(tag)?;
        self.state = State::Tag(tag);
        self.last_tag = tag;
        Ok(())
    }

    fn solve_end(&mut self) -> io::Result<()> {
        self.mapper.write_tag_end(self.last_tag)?;
        self.last_tag = Tag::None;
        Ok(())
    }
}

impl<'a, S: Schema, W: HtmlWriter, R: Read + 'a> Parser for StatefulParser<'a, S, W, R> {
    fn parse_and_write(&mut self) -> Result<(), ParseError> {
        loop {
            match self.tokenizer.next() {
                Some(res) => match res {
                    Ok(token) => {
                        // println!("token: {:?} state: {:?}", token, self.state);
                        self.record.update_nums(token.char_len());

                        match self.state {
                            State::Start | State::Ln => match token {
                                Token::Space => {
                                    self.mapper.write_html_space()?;
                                    self.state = State::PureText;
                                }
                                Token::Tab => {
                                    self.mapper.write_html_tab()?;
                                    self.state = State::PureText;
                                }
                                Token::Ln => {
                                    self.mapper.write_br()?;
                                    self.state = State::Ln;
                                }
                                Token::Tag(tag) => {
                                    self.solve_start(tag)?;
                                }
                                Token::PureText {
                                    data,
                                    char_len: _char_len,
                                } => {
                                    self.mapper.write(data)?;
                                    self.state = State::PureText;
                                }
                            },
                            State::PureText => match token {
                                Token::Space => {
                                    self.mapper.write_html_space()?;
                                }
                                Token::Tab => {
                                    self.mapper.write_html_tab()?;
                                }
                                Token::Ln => {
                                    self.mapper.write_br()?;
                                    self.state = State::Ln;
                                }
                                Token::Tag(tag) => unreachable!(),
                                Token::PureText { data, char_len } => {
                                    self.mapper.write(data)?;
                                }
                            },
                            State::Tag(_) | State::Value => match token {
                                Token::Space => {
                                    self.mapper.write_html_space()?;
                                    self.state = State::Value;
                                }
                                Token::Tab => {
                                    self.mapper.write_html_tab()?;
                                    self.state = State::Value;
                                }
                                Token::Ln => {
                                    let is_header = self.try_solve_header_end()?;

                                    if !is_header {
                                        self.mapper.write_br()?;
                                        self.state = State::BlockValueLn;
                                    }
                                }
                                Token::Tag(_) => unreachable!(),
                                Token::PureText { data, char_len } => {
                                    self.mapper.write(data)?;
                                    self.state = State::Value;
                                }
                            },

                            State::BlockValueLn => match token {
                                Token::Space => {
                                    self.mapper.write_html_space()?;
                                }
                                Token::Tab => {
                                    self.mapper.write_html_tab()?;
                                }
                                Token::Ln => {
                                    self.solve_end()?;
                                    self.state = State::Ln;
                                }
                                Token::Tag(tag) => {
                                    self.solve_end_and_start(tag)?;
                                }
                                Token::PureText { data, char_len } => {
                                    self.mapper.write(data)?;
                                    self.state = State::Value;
                                }
                            },
                        }
                    }
                    Err(e) => return Err(ParseError::IoError(e)),
                },
                None => {
                    self.solve_end()?;
                    self.mapper.write_css()?;
                    return Ok(());
                }
            }
        }
    }
}
