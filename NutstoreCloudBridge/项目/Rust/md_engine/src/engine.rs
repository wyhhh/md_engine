use crate::error_handle::ErrorHandler;
use crate::parser::Parser;

pub struct MarkdownEngine {}

impl MarkdownEngine {
    pub fn new() -> Self {
        Self {}
    }

    // ok 架构基本完成
    pub fn start<P: Parser, H: ErrorHandler>(&mut self, mut p: P, mut h: H) {
        let parse_res = p.parse_and_write();

        match parse_res {
            Ok(_x) => {}
            Err(e) => h.handle_error(e),
        }
    }
}
