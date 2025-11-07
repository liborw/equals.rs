use std::process::{Command, Stdio};

use crate::{
    document::{CodeBlock, CodeBlockUpdate},
    lang::{CodeLine, Language, split_line},
};

const MARKER: &str = "#=";
const COMMENT: &str = "#";

pub struct FendLang;

impl FendLang {
    pub fn new() -> Self {
        Self
    }

    fn evaluate_in_place(&self, blocks: &mut [String]) {
        let originals: Vec<String> = blocks.iter().cloned().collect();
        let parsed: Vec<_> = originals
            .iter()
            .map(|line| split_line(line, MARKER, COMMENT, |_| None))
            .collect();

        let script = build_fend_script(&parsed);
        if script.is_empty() {
            return;
        }

        if let Some(output) = run_fend(&script) {
            for line in output.lines() {
                if let Some(rest) = line.strip_prefix("##RESULT:") {
                    let mut parts = rest.trim_start().splitn(2, ' ');
                    let idx_str = parts.next().unwrap_or_default();
                    let value = parts.next().unwrap_or("").trim();

                    if let Ok(idx) = idx_str.parse::<usize>() {
                        if let Some(target) = blocks.get_mut(idx) {
                            let reconstructed = parsed[idx].reconstruct(value);
                            *target = reconstructed;
                        }
                    }
                }
            }
        }
    }
}

impl Language for FendLang {
    fn name(&self) -> &str {
        "fend"
    }

    fn eval_marker(&self) -> &str {
        MARKER
    }

    fn evaluate(&self, blocks: &[CodeBlock]) -> Vec<CodeBlockUpdate> {
        if blocks.is_empty() {
            return Vec::new();
        }

        let mut working: Vec<String> = blocks.iter().map(|b| b.content.to_string()).collect();
        self.evaluate_in_place(&mut working);

        blocks
            .iter()
            .zip(working.into_iter())
            .filter_map(|(block, new_content)| {
                if block.content == new_content {
                    None
                } else {
                    Some(CodeBlockUpdate {
                        id: block.id,
                        content: new_content,
                    })
                }
            })
            .collect()
    }
}

fn build_fend_script(lines: &[CodeLine]) -> String {
    let mut statements = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        match line {
            CodeLine::Code { code } => {
                if !code.is_empty() {
                    statements.push((*code).to_string());
                }
            }
            CodeLine::Eval { code, .. } | CodeLine::EvalAssignment { code, .. } => {
                if !code.is_empty() {
                    statements.push(format!("print \"##RESULT:{} \"; println ({})", idx, code));
                }
            }
        }
    }
    statements.join("; ")
}

fn run_fend(script: &str) -> Option<String> {
    if script.trim().is_empty() {
        return None;
    }

    let output = Command::new("fend")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{BlockId, CodeBlock};

    fn eval_blocks(blocks: &mut [String]) {
        let lang = FendLang;
        let snapshots: Vec<String> = blocks.iter().cloned().collect();
        let code_blocks: Vec<CodeBlock> = snapshots
            .iter()
            .enumerate()
            .map(|(idx, content)| CodeBlock {
                id: BlockId::new(idx),
                content: content.as_str(),
            })
            .collect();

        let updates = lang.evaluate(&code_blocks);
        for update in updates {
            if let Some(slot) = blocks.get_mut(update.id.index()) {
                *slot = update.content;
            }
        }
    }

    #[test]
    fn evaluates_simple_expression() {
        let mut code_blocks = vec![String::from("2 + 3 #=")];
        eval_blocks(&mut code_blocks);
        assert_eq!(code_blocks[0], "2 + 3 #= 5");
    }

    #[test]
    fn leaves_unmarked_lines() {
        let mut code_blocks = vec![
            String::from("usd = 5"),
            String::from("usd * 2 #="),
            String::from("usd"),
        ];
        eval_blocks(&mut code_blocks);
        assert_eq!(code_blocks[0], "usd = 5");
        assert_eq!(code_blocks[1], "usd * 2 #= 10");
        assert_eq!(code_blocks[2], "usd");
    }
}
