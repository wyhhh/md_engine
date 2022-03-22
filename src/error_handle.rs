use crate::parser::ParseError;

pub trait ErrorHandler {
    fn handle_error(&mut self, e: ParseError);
}

pub struct ErrorHandlerImpl {}

impl ErrorHandlerImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl ErrorHandler for ErrorHandlerImpl {
    fn handle_error(&mut self, e: ParseError) {
        println!("{:?}", e);
        panic!()
    }
}
