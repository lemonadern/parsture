use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Bison grammar explorer (rule expansions)", version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Search for grammar rules by name (substring match by default, use --regex for regex)
    Search {
        /// Path to the yacc/bison grammar file (omit to read from stdin)
        #[arg(long, short = 'f')]
        file: Option<PathBuf>,
        /// Pattern to search for
        pattern: String,
        /// Treat pattern as a regular expression
        #[arg(long, short = 'r')]
        regex: bool,
    },
    /// Show rule structure (exact match only)
    Show {
        /// Path to the yacc/bison grammar file (omit to read from stdin)
        #[arg(long, short = 'f')]
        file: Option<PathBuf>,
        /// Rule name to expand (e.g. expression)
        rule_name: String,
        /// Include %prec directives in output
        #[arg(long, short = 'p')]
        include_prec: bool,
        /// Output as Markdown bullet list
        #[arg(long)]
        md: bool,
    },
}
