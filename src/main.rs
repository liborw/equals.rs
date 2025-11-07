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
    let mut args = Args::parse();

    if args.input.as_deref().map(is_markdown_path).unwrap_or(false) {
        args.markdown = true;
    }

    // --- 1. Read input
    let input_text = if let Some(path) = args.input.as_deref() {
        fs::read_to_string(path)?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    };

    let markdown_guess = if args.markdown {
        detect_markdown_language(&input_text).map(|s| s.to_string())
    } else {
        None
    };

    let language_name = args
        .language
        .clone()
        .or_else(|| {
            args.input
                .as_deref()
                .and_then(|path| guess_language_from_path(Path::new(path)).map(|s| s.to_string()))
        })
        .or(markdown_guess)
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
        "fend" | "fd" => Some("fend"),
        _ => None,
    }
}

fn is_markdown_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|os| os.to_str())
        .map(|ext| matches_ignore_case(ext, &["md", "markdown", "mdown", "mkd"]))
        .unwrap_or(false)
}

fn matches_ignore_case(candidate: &str, choices: &[&str]) -> bool {
    let lower = candidate.to_ascii_lowercase();
    choices.iter().any(|c| lower == *c)
}

fn detect_markdown_language(contents: &str) -> Option<&'static str> {
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("```") {
            let ident = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .collect::<String>()
                .to_ascii_lowercase();
            if ident.is_empty() {
                continue;
            }
            match ident.as_str() {
                "python" => return Some("python"),
                "py" => return Some("python"),
                "numbat" => return Some("numbat"),
                "fend" => return Some("fend"),
                _ => continue,
            }
        }
    }
    None
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
    fn detect_fend_extension() {
        assert_eq!(
            guess_language_from_path(Path::new("notes.fend")),
            Some("fend")
        );
        assert_eq!(
            guess_language_from_path(Path::new("notes.FD")),
            Some("fend")
        );
    }

    #[test]
    fn unknown_extension_returns_none() {
        assert_eq!(guess_language_from_path(Path::new("notes.txt")), None);
    }

    #[test]
    fn detects_markdown_path_variants() {
        assert!(is_markdown_path("guide.md"));
        assert!(is_markdown_path("guide.MarkDown"));
        assert!(is_markdown_path("notes.mdown"));
        assert!(!is_markdown_path("script.py"));
    }

    #[test]
    fn detect_markdown_language_from_fence() {
        let doc = r#"
Some text
```python
print("hi")
```
"#;
        assert_eq!(detect_markdown_language(doc), Some("python"));
    }

    #[test]
    fn detect_markdown_language_ignores_unknown() {
        let doc = r#"
```
no language
```
```lolcode
hi
```
```numbat
let x = 2
```
"#;
        assert_eq!(detect_markdown_language(doc), Some("numbat"));
    }
}
