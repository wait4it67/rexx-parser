#![allow(clippy::print_stderr)]
// use rexx_parser::parser::RexxParser;
mod ast;
mod lexer;
mod parser;
mod lsp;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Lsp,
    Lexer {
        // Path or file to tokenize
        #[arg(short, long)]
        path: String,
    },
    Outline {
        // Path or file to outline
        #[arg(short, long)]
        path: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Lexer { path } => {
            for file in list_files(std::path::Path::new(path)) {
                print_file_lexemes(file);
            }
        }
        Commands::Outline { path } => {
            for file in list_files(std::path::Path::new(path)) {
                println!("File: {}", file.display());
                print_file_outline(file);
            }
        }
        Commands::Lsp => {
            // Note that  we must have our logging only write out to stderr.
            eprintln!("Starting REXX LSP server");
            lsp::run_lsp();
        }
    }
}


fn print_file_outline(path: std::path::PathBuf) {
    let content = std::fs::read_to_string(path).unwrap();
    let mut lexer = lexer::Lexer::new(&content);
    let mut parser = parser::RexxParser::new(&mut lexer);
    let result = parser.parse();
    result.iter().for_each(|x| println!("{:?}", x));
}

fn list_files(path: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();

    if path.is_dir() {
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();
            let entry = entry.path();
            if entry.is_dir() {
                result.extend(list_files(&entry));
            } else {
                result.push(entry);
            }
        }
    } else {
        result.push(path.to_path_buf());
    }
    result
}

fn print_file_lexemes(path: std::path::PathBuf) {
    let content = std::fs::read_to_string(path).unwrap();
    let mut lexer = lexer::Lexer::new(&content);
    let result = lexer.tokenize();
    result.iter().for_each(|x| println!("{:?}", x));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_snippets() {
        use std::fs::File;
        use std::io::Read;
        use std::path::Path;

        let path = Path::new("samples").join("snippets");
        for entry in path.read_dir().unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                let mut file = File::open(entry.path()).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                assert_eq!(contents, rebuilt_from_tokens(&contents));
            }
        }
    }

    fn rebuilt_from_tokens(contents: &str) -> String {
        let mut lexer = lexer::Lexer::new(&contents);
        let lines = lexer.tokenize();
        let mut result = String::new();
        for line in lines {
            for token in line.tokens {
                if token.token_type != lexer::TokenType::Unknown {
                    result.push_str(lexer.get_text(&token));
                }
            }
        }
        result
    }
}
