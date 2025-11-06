use std::process::{Command, Stdio};

use crate::{
    document::{CodeBlock, CodeBlockUpdate},
    lang::{CodeLine, Language, split_line},
};

const MARKER: &str = "#=";
const COMMENT: &str = "#";

pub struct NumbatLang;

impl NumbatLang {
    pub fn new() -> Self {
        Self
    }

    fn evaluate_in_place(&self, blocks: &mut [String]) {
        let mut context: Vec<String> = Vec::new();
        for idx in 0..blocks.len() {
            let original = blocks[idx].clone();
            let parsed = split_line(&original, MARKER, COMMENT, extract_assigned_var);
            match &parsed {
                CodeLine::Code { code } => context.push((*code).to_string()),
                CodeLine::Eval { code, .. } => {
                    let mut sequence = context.clone();
                    sequence.push((*code).to_string());
                    if let Some(result) = run_numbat(&sequence) {
                        let new_line = parsed.reconstruct(result.trim());
                        blocks[idx] = new_line;
                    }
                }
                CodeLine::EvalAssignment { var, code, .. } => {
                    let mut sequence = context.clone();
                    sequence.push((*code).to_string());
                    sequence.push((*var).to_string());
                    if let Some(result) = run_numbat(&sequence) {
                        let new_line = parsed.reconstruct(result.trim());
                        blocks[idx] = new_line;
                    }
                    context.push((*code).to_string());
                }
            }
        }
    }
}

impl Language for NumbatLang {
    fn name(&self) -> &str {
        "numbat"
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

fn run_numbat(expressions: &[String]) -> Option<String> {
    if expressions.is_empty() {
        return None;
    }

    let mut command = Command::new("numbat");
    command
        .arg("--no-config")
        .arg("--no-init")
        .arg("--color")
        .arg("never");

    for expr in expressions {
        command.arg("--expression").arg(expr);
    }

    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

fn extract_assigned_var(code: &str) -> Option<&str> {
    let trimmed = code.trim();
    let rest = trimmed.strip_prefix("let ")?;
    let rest = rest.trim_start();
    let (lhs, _rhs) = rest.split_once('=')?;
    let lhs = lhs.trim();
    if lhs.is_empty() {
        return None;
    }

    let var = lhs.split(':').next().map(str::trim)?;
    if var.is_empty() { None } else { Some(var) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{BlockId, CodeBlock};

    fn eval_blocks(blocks: &mut [String]) {
        let lang = NumbatLang;
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

    fn lines(src: &str) -> Vec<String> {
        src.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|s| s.trim_end().to_string())
            .collect()
    }

    #[test]
    fn evaluates_simple_expression() {
        let mut code_blocks = vec![String::from("2 + 3 #=")];
        eval_blocks(&mut code_blocks);
        assert_eq!(code_blocks[0], "2 + 3 #= 5");
    }

    #[test]
    fn handles_context_across_lines() {
        let mut code_blocks = lines(
            r#"
let x = 2
let y = x + 3 #=
y * 2 #=
"#,
        );

        eval_blocks(&mut code_blocks);

        let expected = lines(
            r#"
let x = 2
let y = x + 3 #= 5
y * 2 #= 10
"#,
        );

        assert_eq!(code_blocks, expected);
    }

    #[test]
    fn preserves_unmarked_lines() {
        let mut code_blocks = lines(
            r#"
let x = 2
let y = 3
x * y
"#,
        );

        let before = code_blocks.clone();
        eval_blocks(&mut code_blocks);
        assert_eq!(before, code_blocks);
    }
}
