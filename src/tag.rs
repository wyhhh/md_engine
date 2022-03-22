use crate::schema::Schema;
use enum_len::EnumLen;

type IsDone = bool;
type Level = u8;

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumLen)]
pub enum Tag {
    None,
    Header(Level),
    BlockQuote,
    TaskList(IsDone),
}

impl Tag {
    pub fn char_len(self) -> u32 {
        match self {
            Tag::None => 0,
            Tag::Header(level) => level as u32 + 1,
            Tag::BlockQuote => 2,
            Tag::TaskList(_x) => 6,
        }
    }

    pub const fn tag_index(self) -> usize {
        match self {
            Tag::None => unreachable!(),
            Tag::Header(1) => 0,
            Tag::Header(2) => 1,
            Tag::Header(3) => 2,
            Tag::Header(4) => 3,
            Tag::Header(5) => 4,
            Tag::Header(6) => 5,
            Tag::BlockQuote => 6,
            Tag::TaskList(true) => 7,
            Tag::TaskList(false) => 8,
            _ => unreachable!(),
        }
    }

    pub fn start_tag<S: Schema>(self, s: &S) -> &str {
        match self {
            Tag::None => unreachable!(),
            Tag::Header(level) => Self::header_start(s, level),
            Tag::BlockQuote => s.block_quote_start(),
            Tag::TaskList(is_done) => {
                if is_done {
                    s.task_list_done_start()
                } else {
                    s.task_list_todo_start()
                }
            }
        }
    }

    pub fn end_tag<S: Schema>(self, s: &S) -> &str {
        match self {
            Tag::None => "",
            Tag::Header(level) => Self::header_end(s, level),
            Tag::BlockQuote => s.block_quote_end(),
            Tag::TaskList(is_done) => {
                if is_done {
                    s.task_list_done_end()
                } else {
                    s.task_list_todo_end()
                }
            }
        }
    }

    pub fn header_start<S: Schema>(s: &S, level: u8) -> &str {
        match level {
            1 => s.h1_start(),
            2 => s.h2_start(),
            3 => s.h3_start(),
            4 => s.h4_start(),
            5 => s.h5_start(),
            6 => s.h6_start(),
            _ => unreachable!(),
        }
    }

    pub fn header_end<S: Schema>(s: &S, level: u8) -> &str {
        match level {
            1 => s.h1_end(),
            2 => s.h2_end(),
            3 => s.h3_end(),
            4 => s.h4_end(),
            5 => s.h5_end(),
            6 => s.h6_end(),
            _ => unreachable!(),
        }
    }
}
