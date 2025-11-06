use crate::{
    document::{Block, Document, Line},
    parser::Parser,
};

pub struct MarkdownParser;

impl MarkdownParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_line(&self, number: usize, line: &str, in_code_block: &mut bool) -> Line {
        if Self::is_fence(line) {
            *in_code_block = !*in_code_block;
            return Line {
                number,
                blocks: vec![Block::Text((0, line.len()), line.to_string())],
            };
        }

        if *in_code_block {
            return Self::parse_fenced_code_line(number, line);
        }

        Self::parse_inline_code_line(number, line)
    }

    fn is_fence(line: &str) -> bool {
        line.trim_start().starts_with("```")
    }

    fn parse_fenced_code_line(number: usize, line: &str) -> Line {
        Line {
            number,
            blocks: vec![Block::Code((0, line.len()), line.to_string())],
        }
    }

    fn parse_inline_code_line(number: usize, line: &str) -> Line {
        let mut blocks = Vec::new();
        let mut text_buf = String::new();
        let mut code_buf = String::new();
        let mut inside_inline = false;
        let mut col: usize = 0;

        for ch in line.chars() {
            if ch == '`' {
                if inside_inline {
                    // End of inline code
                    let start = col.saturating_sub(code_buf.len());
                    let end = col;
                    blocks.push(Block::Code((start, end), code_buf.clone()));
                    code_buf.clear();
                    inside_inline = false;
                    text_buf.push('`'); // preserve closing backtick
                } else {
                    // Start of inline code
                    text_buf.push('`');
                    if !text_buf.is_empty() {
                        let start = col.saturating_sub(text_buf.len());
                        let end = col;
                        blocks.push(Block::Text((start, end), text_buf.clone()));
                        text_buf.clear();
                    }
                    inside_inline = true;
                }
            } else if inside_inline {
                code_buf.push(ch);
            } else {
                text_buf.push(ch);
            }
            col += 1;
        }

        Self::flush_inline_buffers(blocks, text_buf, code_buf, inside_inline, col, number)
    }

    fn flush_inline_buffers(
        mut blocks: Vec<Block>,
        text_buf: String,
        code_buf: String,
        inside_inline: bool,
        col: usize,
        number: usize,
    ) -> Line {
        if inside_inline {
            // Unclosed inline code → merge all into text
            let mut full_text = String::new();
            for block in blocks.drain(..) {
                match block {
                    Block::Text(_, t) | Block::Code(_, t) => full_text.push_str(&t),
                }
            }
            full_text.push_str(&text_buf);
            full_text.push_str(&code_buf);
            blocks.push(Block::Text((0, full_text.len()), full_text));
        } else if !text_buf.is_empty() {
            let start = col.saturating_sub(text_buf.len());
            let end = col;
            blocks.push(Block::Text((start, end), text_buf));
        }

        Line { number, blocks }
    }
}

impl Parser for MarkdownParser {
    fn parse(&self, input: &str) -> Document {
        let mut lines = Vec::new();
        let mut in_code_block = false;

        for (i, raw_line) in input.split_inclusive('\n').enumerate() {
            let number = i + 1;
            let mut line = raw_line.to_string();
            if line.ends_with('\n') {
                line.pop();
            }
            lines.push(self.parse_line(number, &line, &mut in_code_block));
        }

        Document { lines }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Block;

    fn mk_parser() -> MarkdownParser {
        MarkdownParser::new()
    }

    fn assert_text_block_eq(block: &Block, expected: &str) {
        match block {
            Block::Text(_, actual) => assert_eq!(
                actual, expected,
                "❌ Expected text block `{expected}`, got `{actual}`"
            ),
            Block::Code(_, actual) => {
                panic!("❌ Expected text block `{expected}`, but found CODE block `{actual}`")
            }
        }
    }

    fn assert_code_block_eq(block: &Block, expected: &str) {
        match block {
            Block::Code(_, actual) => assert_eq!(
                actual, expected,
                "❌ Expected code block `{expected}`, got `{actual}`"
            ),
            Block::Text(_, actual) => {
                panic!("❌ Expected code block `{expected}`, but found TEXT block `{actual}`")
            }
        }
    }

    #[test]
    fn plain_text_line() {
        let doc = mk_parser().parse("Hello world!");
        assert_eq!(doc.lines.len(), 1);
        let blocks = &doc.lines[0].blocks;
        assert_eq!(blocks.len(), 1);
        assert_text_block_eq(&blocks[0], "Hello world!");
    }

    #[test]
    fn inline_code_single() {
        let doc = mk_parser().parse("This has `inline` code.");
        let blocks = &doc.lines[0].blocks;

        let expected = vec![
            ("text", "This has `"),
            ("code", "inline"),
            ("text", "` code."),
        ];

        assert_eq!(blocks.len(), expected.len());
        for (block, (kind, content)) in blocks.iter().zip(expected) {
            match kind {
                "text" => assert_text_block_eq(block, content),
                "code" => assert_code_block_eq(block, content),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn multiple_inline_codes() {
        let doc = mk_parser().parse("`a` + `b` = `c`");
        let blocks = &doc.lines[0].blocks;

        let expected = vec![
            ("text", "`"),
            ("code", "a"),
            ("text", "` + `"),
            ("code", "b"),
            ("text", "` = `"),
            ("code", "c"),
            ("text", "`"),
        ];

        assert_eq!(blocks.len(), expected.len());
        for (block, (kind, content)) in blocks.iter().zip(expected) {
            match kind {
                "text" => assert_text_block_eq(block, content),
                "code" => assert_code_block_eq(block, content),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn unclosed_inline_code_becomes_text() {
        let doc = mk_parser().parse("This `never closes");
        let blocks = &doc.lines[0].blocks;
        assert_eq!(blocks.len(), 1);
        assert_text_block_eq(&blocks[0], "This `never closes");
    }

    #[test]
    fn fenced_code_block_basic() {
        let src = "```\na = 1\nb = 2\n```";
        let doc = mk_parser().parse(src);

        let lines = &doc.lines;
        assert_eq!(lines.len(), 4);

        assert_text_block_eq(&lines[0].blocks[0], "```");
        assert_code_block_eq(&lines[1].blocks[0], "a = 1");
        assert_code_block_eq(&lines[2].blocks[0], "b = 2");
        assert_text_block_eq(&lines[3].blocks[0], "```");
    }

    #[test]
    fn fenced_code_block_with_language_ignored() {
        let src = "```python\nprint('hi')\n```";
        let doc = mk_parser().parse(src);

        let lines = &doc.lines;
        assert_eq!(lines.len(), 3);

        assert_text_block_eq(&lines[0].blocks[0], "```python");
        assert_code_block_eq(&lines[1].blocks[0], "print('hi')");
        assert_text_block_eq(&lines[2].blocks[0], "```");
    }

    #[test]
    fn reconstruct_roundtrip_inline_and_fenced() {
        let src = r#"
Text before.
```python
x = 1
y = 2
```
Inline `2 + 2` works."#;
        let doc = mk_parser().parse(src);
        let reconstructed = doc.reconstruct();
        assert_eq!(
            reconstructed.trim(),
            src.trim(),
            "❌ Roundtrip failed.\nExpected:\n{}\nGot:\n{}",
            src,
            reconstructed
        );
    }
}
