#![feature(trace_macros)]
use std::borrow::Cow;

pub mod engine;
pub mod error_handle;
pub mod html_writer;
pub mod mapper;
pub mod parser;
pub mod schema;
pub mod tag;
pub mod tokenizer;

pub type CowStr = Cow<'static, str>;
pub const HEADER_TAG: u8 = b'#';
pub const BLOCK_QUOTE_TAG: u8 = b'>';
