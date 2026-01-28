//! Simple CLI for supermarkdown HTML to Markdown conversion.
//!
//! # Usage
//!
//! ```bash
//! # Convert HTML file
//! supermarkdown input.html > output.md
//!
//! # Pipe HTML from stdin
//! curl -s https://example.com | supermarkdown > example.md
//!
//! # With options
//! supermarkdown --heading-style setext --exclude "nav,footer" input.html
//! ```

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process;

use supermarkdown::{convert_with_options, HeadingStyle, LinkStyle, Options};

fn print_help() {
    eprintln!(
        r#"supermarkdown - High-performance HTML to Markdown conversion

USAGE:
    supermarkdown [OPTIONS] [FILE]

    If FILE is omitted or "-", reads from stdin.

OPTIONS:
    -h, --help              Print this help message
    -v, --version           Print version information
    --heading-style <STYLE> Heading style: atx (default) or setext
    --link-style <STYLE>    Link style: inline (default) or referenced
    --code-fence <CHAR>     Code fence character: ` (default) or ~
    --bullet <CHAR>         Bullet marker: - (default), *, or +
    --exclude <SELECTORS>   CSS selectors to exclude (comma-separated)

EXAMPLES:
    # Convert a file
    supermarkdown page.html > page.md

    # Pipe from curl
    curl -s https://example.com | supermarkdown

    # Exclude navigation and ads
    supermarkdown --exclude "nav,.ad,#sidebar" page.html
"#
    );
}

fn print_version() {
    eprintln!("supermarkdown {}", env!("CARGO_PKG_VERSION"));
}

fn parse_args() -> Result<(Options, Option<String>), String> {
    let args: Vec<String> = env::args().collect();
    let mut options = Options::new();
    let mut file_path: Option<String> = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "-v" | "--version" => {
                print_version();
                process::exit(0);
            }
            "--heading-style" => {
                i += 1;
                if i >= args.len() {
                    return Err("--heading-style requires a value".to_string());
                }
                options = match args[i].to_lowercase().as_str() {
                    "atx" => options.heading_style(HeadingStyle::Atx),
                    "setext" => options.heading_style(HeadingStyle::Setext),
                    other => return Err(format!("Unknown heading style: {}", other)),
                };
            }
            "--link-style" => {
                i += 1;
                if i >= args.len() {
                    return Err("--link-style requires a value".to_string());
                }
                options = match args[i].to_lowercase().as_str() {
                    "inline" => options.link_style(LinkStyle::Inline),
                    "referenced" | "reference" => options.link_style(LinkStyle::Referenced),
                    other => return Err(format!("Unknown link style: {}", other)),
                };
            }
            "--code-fence" => {
                i += 1;
                if i >= args.len() {
                    return Err("--code-fence requires a value".to_string());
                }
                let fence = match args[i].as_str() {
                    "`" | "backtick" => '`',
                    "~" | "tilde" => '~',
                    other => return Err(format!("Unknown code fence: {}", other)),
                };
                options = options.code_fence(fence);
            }
            "--bullet" => {
                i += 1;
                if i >= args.len() {
                    return Err("--bullet requires a value".to_string());
                }
                let bullet = match args[i].as_str() {
                    "-" | "dash" => '-',
                    "*" | "asterisk" => '*',
                    "+" | "plus" => '+',
                    other => return Err(format!("Unknown bullet marker: {}", other)),
                };
                options = options.bullet_marker(bullet);
            }
            "--exclude" => {
                i += 1;
                if i >= args.len() {
                    return Err("--exclude requires a value".to_string());
                }
                let selectors: Vec<String> = args[i]
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                options = options.exclude_selectors(selectors);
            }
            arg if arg.starts_with('-') => {
                return Err(format!("Unknown option: {}", arg));
            }
            path => {
                if file_path.is_some() {
                    return Err("Multiple input files not supported".to_string());
                }
                file_path = Some(path.to_string());
            }
        }
        i += 1;
    }

    Ok((options, file_path))
}

fn read_input(file_path: Option<String>) -> io::Result<String> {
    match file_path {
        Some(path) if path != "-" => fs::read_to_string(&path),
        _ => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

fn main() {
    let (options, file_path) = match parse_args() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Run 'supermarkdown --help' for usage information.");
            process::exit(1);
        }
    };

    let html = match read_input(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            process::exit(1);
        }
    };

    let markdown = convert_with_options(&html, &options);

    if let Err(e) = io::stdout().write_all(markdown.as_bytes()) {
        eprintln!("Error writing output: {}", e);
        process::exit(1);
    }
}
