mod cli;
mod ops;

use clap::Parser;
use cli::{Args, Command};
use ops::{extract, search_rules};
use std::fs::read_to_string;
use std::io::{self, Read};

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Search {
            file,
            pattern,
            regex,
        } => {
            let src = match read_source(file.as_ref()) {
                Ok(src) => src,
                Err(err) => fail(err),
            };

            let rule_names = search_rules(&src, &pattern, regex).unwrap_or_else(fail);

            for name in rule_names {
                println!("{}", name);
            }
        }
        Command::Show {
            file,
            rule_name,
            include_prec,
            md,
        } => {
            let src = match read_source(file.as_ref()) {
                Ok(src) => src,
                Err(err) => fail(err),
            };

            let alternatives = extract(&src, &rule_name, include_prec).unwrap_or_else(fail);

            if alternatives.is_empty() {
                eprintln!("(no alternatives found)");
                return;
            }

            for alternative in alternatives {
                if alternative.is_empty() {
                    eprintln!("empty alternative");
                } else if md {
                    println!("- {}", alternative.join(" "));
                } else {
                    println!("{}", alternative.join(" "));
                }
            }
        }
    }
}

fn read_source(file: Option<&std::path::PathBuf>) -> Result<String, String> {
    if let Some(path) = file {
        read_to_string(path).map_err(|e| format!("failed to read source file: {e}"))
    } else {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("failed to read from stdin: {e}"))?;
        Ok(buf)
    }
}

fn fail<T>(msg: String) -> T {
    eprintln!("{msg}");
    std::process::exit(1);
}
