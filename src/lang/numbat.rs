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
        let originals: Vec<String> = blocks.iter().cloned().collect();
        let parsed: Vec<_> = originals
            .iter()
            .map(|line| split_line(line, MARKER, COMMENT, extract_assigned_var))
            .collect();

        let has_eval = parsed.iter().any(|line| {
            matches!(
                line,
                CodeLine::Eval { .. } | CodeLine::EvalAssignment { .. }
            )
        });
        if !has_eval {
            return;
        }

        let script = build_numbat_expressions(&parsed);
        if script.is_empty() {
            return;
        }

        if let Some(output) = run_numbat(&script) {
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

fn build_numbat_expressions(lines: &[CodeLine]) -> Vec<String> {
    let mut expressions = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        match line {
            CodeLine::Code { code } => {
                if !code.is_empty() {
                    expressions.push((*code).to_string());
                }
            }
            CodeLine::Eval { code, .. } => {
                if !code.is_empty() {
                    expressions.push(render_print(idx, code));
                }
            }
            CodeLine::EvalAssignment { code, var, .. } => {
                if !code.is_empty() {
                    expressions.push((*code).to_string());
                }
                expressions.push(render_print(idx, var));
            }
        }
    }
    expressions
}

fn render_print(idx: usize, expr: &str) -> String {
    let escaped = escape_quotes(expr);
    format!("print(\"##RESULT:{} {{{}}}\")", idx, escaped)
}

fn escape_quotes(expr: &str) -> String {
    expr.replace('"', "\\\"")
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
