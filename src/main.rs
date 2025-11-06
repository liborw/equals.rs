use clap::Parser as ClapParser;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

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
    let input_text = if let Some(path) = args.input.as_deref() {
        fs::read_to_string(path)?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    };

    let language_name = args
        .language
        .clone()
        .or_else(|| {
            args.input
                .as_deref()
                .and_then(|path| guess_language_from_path(Path::new(path)).map(|s| s.to_string()))
        })
        .unwrap_or_else(|| "python".to_string());

    let lang: Box<dyn Language> = get_language_spec(&language_name)
        .unwrap_or_else(|| panic!("Unknown language: {language_name}"));

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

fn guess_language_from_path(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "py" | "pyw" => Some("python"),
        "nbt" | "nb" => Some("numbat"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_python_extension() {
        assert_eq!(
            guess_language_from_path(Path::new("script.py")),
            Some("python")
        );
        assert_eq!(
            guess_language_from_path(Path::new("script.PYW")),
            Some("python")
        );
    }

    #[test]
    fn detect_numbat_extension() {
        assert_eq!(
            guess_language_from_path(Path::new("calc.nbt")),
            Some("numbat")
        );
        assert_eq!(
            guess_language_from_path(Path::new("calc.NB")),
            Some("numbat")
        );
    }

    #[test]
    fn unknown_extension_returns_none() {
        assert_eq!(guess_language_from_path(Path::new("notes.txt")), None);
    }
}
