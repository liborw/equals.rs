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
mod tests {}
