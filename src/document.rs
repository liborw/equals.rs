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

    pub fn code_blocks_mut(&mut self) -> Vec<&mut String> {
        let mut refs = Vec::new();
        for line in &mut self.lines {
            for block in &mut line.blocks {
                if let Block::Code(_, code) = block {
                    refs.push(code);
                }
            }
        }
        refs
    }

    pub fn evaluate_with<F>(&mut self, mut evaluator: F)
    where
        F: FnMut(&mut [String]),
    {
        // extract code block texts
        let mut code_texts: Vec<String> = self
            .lines
            .iter()
            .flat_map(|line| {
                line.blocks.iter().filter_map(|b| match b {
                    Block::Code(_, code) => Some(code.clone()),
                    _ => None,
                })
            })
            .collect();

        // evaluate on contiguous slice
        evaluator(&mut code_texts);

        // write evaluated code blocks back
        let mut i = 0;
        for line in &mut self.lines {
            for block in &mut line.blocks {
                if let Block::Code(_, code) = block {
                    *code = code_texts[i].clone();
                    i += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
}
