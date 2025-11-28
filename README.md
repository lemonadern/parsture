# parsture

A CLI tool for exploring Bison/Yacc grammar files and understanding rule structures.

## Overview

parsture provides the following features:

- **Rule name search**: Search for rule names using substring matching or regular expressions
- **Rule structure display**: List right-hand side alternatives for a specified rule

## Usage

parsture uses a subcommand-based interface.

### Search for rule names (`search`)

Search for rule names. By default, substring matching is used. Use the `--regex` option for regular expression matching.

```bash
parsture search [OPTIONS] <PATTERN>
```

#### Options

- `-f, --file <FILE>`: Path to the grammar file to parse (omit to read from stdin)
- `-r, --regex`: Treat pattern as a regular expression
- `-h, --help`: Print help

#### Arguments

- `<PATTERN>`: Pattern to search for (string or regular expression)

#### Examples

```bash
# Substring match (default)
parsture search -f gram.y expr

# Regular expression search
parsture search --regex -f gram.y "^expr"

# Read from stdin
cat gram.y | parsture search expr
```

#### Output

Prints matched rule names one per line, in the order they appear in the file (top to bottom).

```
expression
expression_list
expression_statement
```

### Show rule structure (`show`)

List right-hand side alternatives for a specified rule. Only exact match is supported.

```bash
parsture show [OPTIONS] <RULE_NAME>
```

#### Options

- `-f, --file <FILE>`: Path to the grammar file to parse (omit to read from stdin)
- `-p, --include-prec`: Include `%prec` directives in output
- `--md`: Output as Markdown bullet list
- `-h, --help`: Print help

#### Arguments

- `<RULE_NAME>`: Rule name to display (exact match)

#### Examples

```bash
# Default format
parsture show -f gram.y expression

# Markdown format
parsture show --md -f gram.y expression

# Include %prec directives
parsture show --include-prec -f gram.y expression

# Read from stdin
cat gram.y | parsture show expression
```

#### Output Format

**Default format**:
```
token1 token2 token3
token4 token5
```

**Markdown format** (with `--md` option):
```markdown
- token1 token2 token3
- token4 token5
```

Action blocks (`{ ... }`) are ignored, and symbol sequences separated by `|` are collected and output.

## Additional Utilities

### `list_rules` example

List all rule names and their definition start lines in the grammar.

```bash
cargo run --example list_rules -- gram.y
```

Example output:
```
rules: 1234, errors: 0, span: (0, 0)-(12345, 0)
expression	100
statement	150
...
```

## Known Limitations

- Rules must be separated by semicolons. Consecutive rules without semicolons cannot be parsed.
- If parsing errors occur, a warning `note: CST includes errors; result may be incomplete` will be displayed.
