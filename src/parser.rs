use crate::document::{Block, Document, Line};

pub trait Parser {
    fn parse(&self, input: &str) -> Document;
}

pub struct PlainParser {}

impl Parser for PlainParser {
    fn parse(&self, input: &str) -> Document {
        let lines = input
            .lines()
            .enumerate()
            .map(|(i, line_text)| {
                // Compute column range — the entire line is a code block
                let start_col = 0;
                let end_col = line_text.len();
                Line {
                    number: i + 1,
                    blocks: vec![Block::Code((start_col, end_col), line_text.to_string())],
                }
            })
            .collect::<Vec<_>>();

        Document { lines }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Block;

    #[test]
    fn test_plain_parser_roundtrip() {
        let src = "x = 1\ny = 2\n x + y #= 3";
        let parser = PlainParser {};

        let doc = parser.parse(src);
        let reconstructed = doc.reconstruct();

        assert_eq!(reconstructed.trim(), src.trim());

        // verify blocks
        for line in &doc.lines {
            assert_eq!(line.blocks.len(), 1);
            match &line.blocks[0] {
                Block::Code(_, code) => {
                    assert!(!code.is_empty());
                }
                _ => panic!("expected code block"),
            }
        }
    }
}
