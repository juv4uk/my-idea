use my_lisp::{eval_parsed_expressions, parse, EvalResult, Expr, LanguageError, Session};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

/// Remaps a concatenated string offset back to the original source file offset.
fn remap_offset(offset: usize, maps: &[(usize, usize, usize)]) -> usize {
    for &(concat_start, concat_end, orig_start) in maps {
        if offset >= concat_start && offset < concat_end {
            return orig_start + (offset - concat_start);
        }
    }
    // Fallback: if offset is exactly at the end of the last block
    if let Some(&(concat_start, concat_end, orig_start)) = maps.last() {
        if offset == concat_end {
            return orig_start + (offset - concat_start);
        }
    }
    offset
}

fn remap_error(mut error: LanguageError, maps: &[(usize, usize, usize)]) -> LanguageError {
    error.span.start = remap_offset(error.span.start, maps);
    error.span.end = remap_offset(error.span.end, maps);
    error
}

pub fn eval_literate(source: &str, mode: &str, session: &mut Session) -> Result<(EvalResult, Vec<Expr>), LanguageError> {
    let mut concatenated = String::new();
    let mut offset_maps = Vec::new();

    let parser = Parser::new(source).into_offset_iter();
    let mut in_my_lisp_block = false;

    for (event, range) in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) if lang.as_ref() == "my-lisp" => {
                in_my_lisp_block = true;
            }
            Event::End(TagEnd::CodeBlock) if in_my_lisp_block => {
                in_my_lisp_block = false;
            }
            Event::Text(text) if in_my_lisp_block => {
                let concat_start = concatenated.len();
                concatenated.push_str(&text);
                let concat_end = concatenated.len();
                offset_maps.push((concat_start, concat_end, range.start));
            }
            _ => {}
        }
    }

    let is_literate = mode == "markdown";

    if !is_literate {
        // Pure Lisp mode: evaluate the entire source
        concatenated = source.to_string();
        offset_maps.push((0, source.len(), 0));
    } else if offset_maps.is_empty() {
        // Literate mode but no my-lisp blocks found
        return Ok((
            EvalResult {
                value: my_lisp::Value::Nil,
                output: vec!["No my-lisp code blocks found in markdown document.".to_string()],
            },
            vec![]
        ));
    }

    let forms = parse(&concatenated).map_err(|e| remap_error(e, &offset_maps))?;
    
    // Evaluate core bootstrap first
    my_lisp::eval_program(include_str!("../../../lib/core.my"), session).map_err(|e| remap_error(e, &offset_maps))?;
    
    let result = eval_parsed_expressions(&forms, session).map_err(|e| remap_error(e, &offset_maps))?;
    Ok((result, forms))
}
