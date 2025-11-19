use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(about = "Bison grammar explorer (rule expansions)", version)]
pub struct Args {
    /// Path to the yacc/bison grammar file (omit to read from stdin)
    #[arg(long, short = 'f')]
    pub file: Option<PathBuf>,
    /// Rule name to expand (e.g. expression)
    pub rule_name: String,
    /// Include %prec directives in output
    #[arg(long, short = 'p')]
    pub include_prec: bool,
    /// Output as Markdown bullet list
    #[arg(long)]
    pub md: bool,
}
