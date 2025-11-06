use clap::Parser as ClapParser;
use std::fs;
use std::io::{self, Read};

mod document;
mod lang;
mod markdown;
mod parser;

use crate::lang::{Language, get_language_spec};
use crate::markdown::MarkdownParser;
use crate::parser::{Parser, PlainParser};

/// equals — evaluate code inside text or markdown files
#[derive(ClapParser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Input file (if not provided, reads from stdin)
    #[arg(short, long)]
    input: Option<String>,

    /// Output file (if not provided, prints to stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Force language (optional, e.g. "python", "numbat")
    #[arg(short, long)]
    language: Option<String>,

    /// Parse as Markdown (if not set, uses plain text parser)
    #[arg(short = 'm', long)]
    markdown: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // --- 1. Read input
    let input_text = if let Some(path) = args.input {
        fs::read_to_string(path)?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    };

    // language
    let lang: Box<dyn Language> =
        get_language_spec(&args.language.unwrap_or("python".into())).expect("Unknown language");

    // --- 2. Parse document
    let parser: Box<dyn Parser> = if args.markdown {
        Box::new(MarkdownParser::new())
    } else {
        Box::new(PlainParser {})
    };
    let mut doc = parser.parse(&input_text);
    doc.evaluate_with(|blocks| lang.evaluate(blocks));
    let output_text = doc.reconstruct();

    // --- 6. Write output
    if let Some(path) = args.output {
        fs::write(path, output_text)?;
    } else {
        print!("{output_text}");
    }

    Ok(())
}
