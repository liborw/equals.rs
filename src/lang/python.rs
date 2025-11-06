use std::io::Write;
use std::process::{Command, Stdio};

use crate::{
    document::{CodeBlock, CodeBlockUpdate},
    lang::{CodeLine, Language, split_line},
};

pub struct PythonLang;

const MARKER: &str = "#=";
const COMMENT: &str = "#";

impl PythonLang {
    pub fn new() -> Self {
        Self
    }
}

impl Language for PythonLang {
    fn name(&self) -> &str {
        "python"
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

impl PythonLang {
    fn evaluate_in_place(&self, input: &mut [String]) {
        let cloned_inputs: Vec<String> = input.to_vec();

        let lines: Vec<_> = cloned_inputs
            .iter()
            .map(|s| split_line(s, self.eval_marker(), COMMENT, extract_assigned_var))
            .collect();

        let script = build_python_script(&lines);
        let output = run_python(&script);

        for line in output.lines() {
            if let Some(rest) = line.strip_prefix("##RESULT:") {
                let mut parts = rest.split_whitespace();
                if let Some(idx_str) = parts.next() {
                    if let Ok(idx) = idx_str.parse::<usize>() {
                        let value = parts.collect::<Vec<_>>().join(" ");
                        if let Some(s) = input.get_mut(idx) {
                            let new_line = lines[idx].reconstruct(&value);
                            *s = new_line;
                        }
                    }
                }
            }
        }
    }
}

// Check whether a line looks like an assignment statement
fn is_assignment(code: &str) -> bool {
    code.contains('=') && !code.contains("==") && !code.contains("!=")
}

// Extract the variable name on the left-hand side of an assignment
fn extract_assigned_var(code: &str) -> Option<&str> {
    if !is_assignment(code) {
        return None;
    }

    code.split('=')
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn build_python_script(input: &[CodeLine]) -> String {
    input
        .iter()
        .enumerate()
        .map(|(i, line)| match line {
            CodeLine::Code { code } => code.to_string(),
            CodeLine::Eval { code, .. } => format!("print('##RESULT:{}', {})", i, code),
            CodeLine::EvalAssignment { var, code, .. } => {
                format!("{}\nprint('##RESULT:{}', {})", code, i, var)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// Run the generated Python code and return stdout
fn run_python(script: &str) -> String {
    let output = Command::new("python3")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(script.as_bytes()).ok();
            }
            child.wait_with_output()
        })
        .expect("failed to run python");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{BlockId, CodeBlock};

    // Simple wrapper to call PythonLang::evaluate on &mut [String]
    fn eval_blocks(blocks: &mut [String]) {
        let lang = PythonLang;
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

    // Helper to split a multiline code snippet into Vec<String> lines
    fn lines(code: &str) -> Vec<String> {
        code.lines()
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
    fn preserves_comment_after_result() {
        let mut code_blocks = vec![String::from("10 + 2 #= 23 # expected")];
        eval_blocks(&mut code_blocks);
        assert_eq!(code_blocks[0], "10 + 2 #= 12 # expected");
    }

    #[test]
    fn handles_multiple_marked_lines() {
        let mut code_blocks = lines(
            r#"
x = 2
y = 3
x + y #=
x * y #=
"#,
        );

        eval_blocks(&mut code_blocks);

        let expected = lines(
            r#"
x = 2
y = 3
x + y #= 5
x * y #= 6
"#,
        );

        assert_eq!(
            code_blocks, expected,
            "\nExpected evaluated code blocks to match computed results.\nGot:\n{:#?}",
            code_blocks
        );
    }

    #[test]
    fn ignores_unmarked_lines() {
        let mut code_blocks = lines(
            r#"
x = 1
y = 2
z = 3
x + y + z
"#,
        );

        let before = code_blocks.clone();
        eval_blocks(&mut code_blocks);
        assert_eq!(
            before, code_blocks,
            "Unmarked lines should remain unchanged"
        );
    }

    #[test]
    fn complex_block_with_comments_and_results() {
        let mut code_blocks = lines(
            r#"
a = 4
b = 5
a * b #= 20 # precomputed
a + b #=
(a + b) * 2 #=
"#,
        );

        eval_blocks(&mut code_blocks);

        let expected = lines(
            r#"
a = 4
b = 5
a * b #= 20 # precomputed
a + b #= 9
(a + b) * 2 #= 18
"#,
        );

        assert_eq!(
            code_blocks, expected,
            "\nComplex block did not evaluate as expected.\nGot:\n{:#?}",
            code_blocks
        );
    }

    #[test]
    fn produces_valid_python_script() {
        let mut code_blocks = lines(
            r#"
values = [1, 2, 3]
sum(values) #=
"#,
        );

        eval_blocks(&mut code_blocks);

        let expected = lines(
            r#"
values = [1, 2, 3]
sum(values) #= 6
"#,
        );

        assert_eq!(
            code_blocks, expected,
            "\nPython script should have produced correct result.\nGot:\n{:#?}",
            code_blocks
        );
    }

    #[test]
    fn assignment_in_equals_line() {
        let mut code_blocks = lines(
            r#"
a = 1
b = 2
c = a + b #=
"#,
        );

        eval_blocks(&mut code_blocks);

        let expected = lines(
            r#"
a = 1
b = 2
c = a + b #= 3
"#,
        );

        assert_eq!(
            code_blocks, expected,
            "\nAssignment line did not evaluate as expected.\nGot:\n{:#?}",
            code_blocks
        );
    }
}
