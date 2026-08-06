use pulldown_cmark::{Parser, Event, Tag, CodeBlockKind};

fn main() {
    let source = "# Literate Lisp\nThis is a literate program.\n```my-lisp\n(def x 10)\n(* x 2)\n```\nIt ignores non-code blocks.";
    let parser = Parser::new(source);
    for event in parser {
        println!("{:?}", event);
    }
}
