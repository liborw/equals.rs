use std::collections::HashMap;

#[derive(Debug)]
pub struct Document {
    pub lines: Vec<Line>,
}

#[derive(Debug)]
pub struct Line {
    pub number: usize,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub enum Block {
    Text((usize, usize), String),
    Code((usize, usize), String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(usize);

impl BlockId {
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CodeBlock<'a> {
    pub id: BlockId,
    pub content: &'a str,
}

#[derive(Debug, Clone)]
pub struct CodeBlockUpdate {
    pub id: BlockId,
    pub content: String,
}

impl Document {
    pub fn reconstruct(&self) -> String {
        self.lines
            .iter()
            .map(|line| {
                // Sort blocks by start column to ensure correct ordering
                let mut blocks = line.blocks.clone();
                blocks.sort_by_key(|b| match b {
                    Block::Text((start, _), _) => *start,
                    Block::Code((start, _), _) => *start,
                });

                blocks
                    .iter()
                    .map(|block| match block {
                        Block::Text(_, text) => text.clone(),
                        Block::Code(_, code) => code.clone(),
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn evaluate_with<F>(&mut self, evaluator: F)
    where
        F: FnOnce(&[CodeBlock]) -> Vec<CodeBlockUpdate>,
    {
        let mut extracted: Vec<(BlockId, String)> = Vec::new();

        for line in &mut self.lines {
            for block in &mut line.blocks {
                if let Block::Code(_, code) = block {
                    let id = BlockId::new(extracted.len());
                    extracted.push((id, std::mem::take(code)));
                }
            }
        }

        if extracted.is_empty() {
            return;
        }

        let view: Vec<CodeBlock> = extracted
            .iter()
            .map(|(id, content)| CodeBlock {
                id: *id,
                content: content.as_str(),
            })
            .collect();

        let updates = evaluator(&view);
        let mut updates_map: HashMap<BlockId, String> = HashMap::new();
        for update in updates {
            updates_map.insert(update.id, update.content);
        }

        let mut extracted_iter = extracted.into_iter();
        for line in &mut self.lines {
            for block in &mut line.blocks {
                if let Block::Code(_, code) = block {
                    let (id, original) = extracted_iter
                        .next()
                        .expect("mismatched number of code blocks during evaluation");
                    if let Some(updated) = updates_map.remove(&id) {
                        *code = updated;
                    } else {
                        *code = original;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn code_line(number: usize, content: &str) -> Line {
        Line {
            number,
            blocks: vec![Block::Code((0, content.len()), content.to_string())],
        }
    }

    #[test]
    fn evaluate_with_applies_partial_updates() {
        let mut doc = Document {
            lines: vec![code_line(1, "x = 1"), code_line(2, "x + 1 #=")],
        };

        doc.evaluate_with(|blocks| {
            assert_eq!(blocks.len(), 2);
            assert_eq!(blocks[0].id.index(), 0);
            assert_eq!(blocks[1].id.index(), 1);
            assert_eq!(blocks[1].content, "x + 1 #=");

            vec![CodeBlockUpdate {
                id: blocks[1].id,
                content: "x + 1 #= 2".into(),
            }]
        });

        let line1 = &doc.lines[0].blocks[0];
        let line2 = &doc.lines[1].blocks[0];

        match line1 {
            Block::Code(_, text) => assert_eq!(text, "x = 1"),
            _ => panic!("expected code block for line 1"),
        }

        match line2 {
            Block::Code(_, text) => assert_eq!(text, "x + 1 #= 2"),
            _ => panic!("expected code block for line 2"),
        }
    }

    #[test]
    fn evaluate_with_keeps_original_when_no_updates() {
        let mut doc = Document {
            lines: vec![code_line(1, "print('hi')")],
        };

        doc.evaluate_with(|blocks| {
            assert_eq!(blocks.len(), 1);
            assert_eq!(blocks[0].content, "print('hi')");
            Vec::new()
        });

        match &doc.lines[0].blocks[0] {
            Block::Code(_, text) => assert_eq!(text, "print('hi')"),
            _ => panic!("expected code block to remain unchanged"),
        }
    }

    #[test]
    fn evaluate_with_skips_when_no_code_blocks() {
        let mut doc = Document {
            lines: vec![Line {
                number: 1,
                blocks: vec![Block::Text((0, 4), "text".into())],
            }],
        };

        let called = Cell::new(false);
        doc.evaluate_with(|_| {
            called.set(true);
            Vec::new()
        });

        assert!(
            !called.get(),
            "evaluator should not be invoked when no code blocks exist"
        );

        match &doc.lines[0].blocks[0] {
            Block::Text(_, text) => assert_eq!(text, "text"),
            _ => panic!("text block should remain untouched"),
        }
    }
}
