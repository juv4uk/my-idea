use my_lisp::{eval_parsed_expressions, parse, Session};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut session = Session::default();
    
    // Load standard library
    let core_lib = include_str!("../../../lib/core.my");
    if let Ok(core_ast) = parse(core_lib) {
        let _ = eval_parsed_expressions(&core_ast, &mut session);
    }

    if args.len() > 1 {
        // Run file
        let filename = &args[1];
        match fs::read_to_string(filename) {
            Ok(source) => {
                match parse(&source) {
                    Ok(ast) => {
                        match eval_parsed_expressions(&ast, &mut session) {
                            Ok(result) => {
                                for out in result.output {
                                    println!("{}", out);
                                }
                                println!("{}", result.value);
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Parse error: {}", e);
                        process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        }
    } else {
        // REPL mode
        println!("my-lisp REPL v0.1.0 (pure Rust)");
        println!("Press Ctrl-C or Ctrl-D to exit.");

        let mut rl = DefaultEditor::new().unwrap();
        
        loop {
            let readline = rl.readline("my-lisp> ");
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    
                    let _ = rl.add_history_entry(line);
                    
                    match parse(line) {
                        Ok(ast) => {
                            match eval_parsed_expressions(&ast, &mut session) {
                                Ok(result) => {
                                    for out in result.output {
                                        println!("{}", out);
                                    }
                                    println!("{}", result.value);
                                }
                                Err(e) => {
                                    eprintln!("Error: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Parse error: {}", e);
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl-C
                    break;
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl-D
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }
    }
}
