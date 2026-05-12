use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::process;

use giulia_lexer::lex;
use giulia_parser::parse;
use giulia_interpreter::run;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("repl") => repl(),
        Some("check") => {
            if let Some(path) = args.get(2) {
                check_file(path);
            } else {
                eprintln!("usage: giulia check <file.gcrl>");
            }
        }
        Some("run") => {
            if let Some(path) = args.get(2) {
                run_file(path);
            } else {
                eprintln!("usage: giulia run <file.gcrl>");
            }
        }
        Some("--version") | Some("-v") => {
            println!("giulia {}", env!("CARGO_PKG_VERSION"));
        }
        _ => print_help(),
    }
}

fn print_help() {
    println!("Giulia CLI");
    println!();
    println!("Commands:");
    println!("  giulia repl                Interactive REPL (lex + parse)");
    println!("  giulia check <file.gcrl>    Tokenize, parse, and print AST");
    println!("  giulia run <file.gcrl>      Parse and execute a GCRL script");
    println!("  giulia --version, -v       Print version");
}

fn repl() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();

    loop {
        print!("> ");
        stdout.flush().ok();
        input.clear();

        if stdin.lock().read_line(&mut input).ok().map_or(true, |n| n == 0) {
            break;
        }

        let line = input.trim();
        if line.is_empty() {
            continue;
        }
        if line == ":quit" || line == ":q" {
            break;
        }

        process_source(line);
    }
}

fn check_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading file: {e}");
            return;
        }
    };
    process_source(&source);
}

fn run_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading file: {e}");
            process::exit(1);
        }
    };

    let tokens = match lex(&source) {
        Ok(tokens) => tokens,
        Err(errors) => {
            for err in &errors {
                eprintln!("  lex error: {err}");
            }
            process::exit(1);
        }
    };

    let program = match parse(tokens) {
        Ok(program) => program,
        Err(errors) => {
            for err in &errors {
                eprintln!("  parse error: {err}");
            }
            process::exit(1);
        }
    };

    match run(&program, &source) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("  runtime error: {err}");
            process::exit(1);
        }
    }
}

fn process_source(source: &str) {
    let tokens = match lex(source) {
        Ok(tokens) => tokens,
        Err(errors) => {
            for err in &errors {
                eprintln!("  lex error: {err}");
            }
            return;
        }
    };

    println!("── Tokens ──");
    for st in &tokens {
        let token_str = format!("{:?}", st.token);
        let padded = format!("{:width$}", token_str, width = 30);
        println!(
            "  {}  line={:>3} col={:>3}  pos={}..{}",
            padded, st.line, st.column, st.span.start, st.span.end,
        );
    }

    match parse(tokens) {
        Ok(program) => {
            println!("\n── AST ──");
            println!("{:#?}", program);
        }
        Err(errors) => {
            println!("\n── Parse Errors ──");
            for err in &errors {
                eprintln!("  parse error: {err}");
            }
        }
    }
}
