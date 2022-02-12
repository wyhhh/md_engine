use md_engine::engine::MarkdownEngine;
use md_engine::error_handle::ErrorHandlerImpl;
use md_engine::html_writer::HtmlWriterImpl;
use md_engine::parser::StatefulParser;
use md_engine::schema::DefaultSchema;
use std::fs::File;
use std::time::Instant;

fn main() {
    let now = Instant::now();

    let schema = DefaultSchema;
    let writer = HtmlWriterImpl::new(File::create("test.html").unwrap());
    let open = File::open("test.md").unwrap();
    let parser = StatefulParser::new(open, schema, writer);
    let error_handler = ErrorHandlerImpl::new();

    let mut engine = MarkdownEngine::new();
    engine.start(parser, error_handler);

    println!("ok. cost time: {:?}", now.elapsed());
}
