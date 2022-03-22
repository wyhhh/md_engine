# Markdown Processor
Blazingly fast processer with Markdown format, NO ALLOCATION in the process, one pass resolving.

# Todo
✨ Headers<br>
✨ Todo List<br>
✨ Quote
- [ ] Code
- [ ] etc...

# Example

```rust
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
```


