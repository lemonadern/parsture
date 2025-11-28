use std::path::PathBuf;

use clap::{ArgGroup, Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Bison grammar explorer (rule expansions)", version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Search for grammar rules (searches both left-hand side and right-hand side by default, use --regex for regex)
    #[command(group(
        ArgGroup::new("search_target")
            .args(&["lhs", "rhs"])
            .multiple(false)
    ))]
    Search {
        /// Path to the yacc/bison grammar file (omit to read from stdin)
        #[arg(long, short = 'f')]
        file: Option<PathBuf>,
        /// Pattern to search for
        pattern: String,
        /// Treat pattern as a regular expression
        #[arg(long, short = 'r')]
        regex: bool,
        /// Search in left-hand side (rule names) only
        #[arg(long)]
        lhs: bool,
        /// Search in right-hand side (rule bodies) only
        #[arg(long)]
        rhs: bool,
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
