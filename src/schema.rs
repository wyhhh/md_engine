pub trait Schema {
    fn h1_start(&self) -> &str;
    fn h1_end(&self) -> &str;
    fn h2_start(&self) -> &str;
    fn h2_end(&self) -> &str;
    fn h3_start(&self) -> &str;
    fn h3_end(&self) -> &str;
    fn h4_start(&self) -> &str;
    fn h4_end(&self) -> &str;
    fn h5_start(&self) -> &str;
    fn h5_end(&self) -> &str;
    fn h6_start(&self) -> &str;
    fn h6_end(&self) -> &str;
    fn block_quote_start(&self) -> &str;
    fn block_quote_end(&self) -> &str;
    fn task_list_done_start(&self) -> &str;
    fn task_list_done_end(&self) -> &str;
    fn task_list_todo_start(&self) -> &str;
    fn task_list_todo_end(&self) -> &str;
    fn code_block_start(&self) -> &str;
    fn code_block_end(&self) -> &str;

    fn h1_css(&self) -> &str;
    fn h2_css(&self) -> &str;
    fn h3_css(&self) -> &str;
    fn h4_css(&self) -> &str;
    fn h5_css(&self) -> &str;
    fn h6_css(&self) -> &str;
    fn block_quote_css(&self) -> &str;
    fn task_list_done_css(&self) -> &str;
    fn task_list_todo_css(&self) -> &str;
    fn code_block_css(&self) -> &str;

    fn css_tag_start(&self) -> &str {
        "<style>"
    }
    fn css_tag_end(&self) -> &str {
        "</style>"
    }
}

pub trait SyntaxHighlight {}

pub struct DefaultSchema;

impl Schema for DefaultSchema {
    fn h1_start(&self) -> &str {
        "<h1>"
    }

    fn h1_end(&self) -> &str {
        "</h1>"
    }

    fn h2_start(&self) -> &str {
        "<h2>"
    }

    fn h2_end(&self) -> &str {
        "</h2>"
    }

    fn h3_start(&self) -> &str {
        "<h3>"
    }

    fn h3_end(&self) -> &str {
        "</h3>"
    }

    fn h4_start(&self) -> &str {
        "<h4>"
    }

    fn h4_end(&self) -> &str {
        "</h4>"
    }

    fn h5_start(&self) -> &str {
        "<h5>"
    }

    fn h5_end(&self) -> &str {
        "</h5>"
    }

    fn h6_start(&self) -> &str {
        "<h6>"
    }

    fn h6_end(&self) -> &str {
        "</h6>"
    }

    fn block_quote_start(&self) -> &str {
        r#"<div class="block-quote">"#
    }

    fn block_quote_end(&self) -> &str {
        "</div>"
    }

    fn task_list_done_start(&self) -> &str {
        r#"<div class="task-list-done"></div><span class="task-list-done-text">"#
    }

    fn task_list_done_end(&self) -> &str {
        "</span>"
    }

    fn task_list_todo_start(&self) -> &str {
        r#"<div class="task-list-todo"></div><span class="task-list-todo-text">"#
    }

    fn task_list_todo_end(&self) -> &str {
        "</span>"
    }

    fn h1_css(&self) -> &str {
        ""
    }

    fn h2_css(&self) -> &str {
        ""
    }

    fn h3_css(&self) -> &str {
        ""
    }

    fn h4_css(&self) -> &str {
        ""
    }

    fn h5_css(&self) -> &str {
        ""
    }

    fn h6_css(&self) -> &str {
        ""
    }

    fn block_quote_css(&self) -> &str {
        r#".block-quote {
			margin-top: 5px;
			border-left: 2px solid #666666;
			padding-left: 10px; 
			color: #888888
		}"#
    }

    fn task_list_done_css(&self) -> &str {
        r#".task-list-done {
			display:inline-block;
			border: black solid 1px;
			width: 10px;
			height: 10px
		}

		.task-list-done-text {
			font-style: italic
		}"#
    }

    fn task_list_todo_css(&self) -> &str {
        r#".task-list-todo {
			display:inline-block;
			background-color: black;
			width: 12px;
			height: 12px
		}

		.task-list-todo-text {
			font-style: italic
		}"#
    }

    fn code_block_start(&self) -> &str {
        r#"<div class="code-block">"#
    }

    fn code_block_end(&self) -> &str {
        "</div>"
    }

    fn code_block_css(&self) -> &str {
        r#".code-block {
			background-color: #999999
		}"#
    }
}
