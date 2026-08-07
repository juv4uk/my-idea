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
        let arg = &args[1];
        
        if arg == "--version" || arg == "-V" || arg == "-v" {
            println!("my-lisp {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        
        if arg == "--help" || arg == "-h" {
            println!("Usage: my-lisp [file]");
            println!("If no file is provided, starts the REPL.");
            println!("\nOptions:");
            println!("  -V, --version  Print version information");
            println!("  -h, --help     Print help information");
            return;
        }

        // Run file
        let filename = arg;
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
        println!("my-lisp REPL v{} (pure Rust)", env!("CARGO_PKG_VERSION"));
        println!("Press Ctrl-C or Ctrl-D to exit.");

        // rustyline can fail to init on an unusual terminal (e.g. no TTY); report it
        // cleanly instead of panicking, so a redirected/CI invocation exits with a message.
        // rustyline може не ініціалізуватися на нетиповому терміналі (напр. без TTY);
        // повідомляємо про це чисто замість паніки, щоб перенаправлений/CI-запуск завершився з повідомленням.
        // rustyline kann bei einem ungewöhnlichen Terminal (z. B. ohne TTY) fehlschlagen;
        // das wird sauber gemeldet statt einen Panic auszulösen, damit ein umgeleiteter/CI-Aufruf mit Meldung endet.
        let mut rl = match DefaultEditor::new() {
            Ok(editor) => editor,
            Err(err) => {
                eprintln!("Error: could not start the REPL line editor: {err}");
                process::exit(1);
            }
        };

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
