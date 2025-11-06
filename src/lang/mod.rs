use std::fmt::Debug;

use crate::{
    document::{CodeBlock, CodeBlockUpdate},
    lang::{numbat::NumbatLang, python::PythonLang},
};

pub mod numbat;
pub mod python;

pub trait Language {
    fn name(&self) -> &str;
    fn eval_marker(&self) -> &str;
    fn evaluate(&self, blocks: &[CodeBlock]) -> Vec<CodeBlockUpdate>;
}

impl Debug for dyn Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Language(name = {}, marker = {})",
            self.name(),
            self.eval_marker()
        )
    }
}

pub fn get_language_spec(lang_str: &str) -> Option<Box<dyn Language>> {
    match lang_str {
        "python" => Some(Box::new(PythonLang::new())),
        "numbat" => Some(Box::new(NumbatLang::new())),
        _ => None,
    }
}

#[derive(Debug, PartialEq)]
pub enum CodeLine<'a> {
    Code {
        code: &'a str,
    },
    Eval {
        code: &'a str,
        marker: &'a str,
        result: Option<&'a str>,
        comment: Option<&'a str>,
    },
    EvalAssignment {
        var: &'a str,
        code: &'a str,
        marker: &'a str,
        result: Option<&'a str>,
        comment: Option<&'a str>,
    },
}

impl<'a> CodeLine<'a> {
    /// Reconstructs the line with a new `result`.
    pub fn reconstruct(&self, result: &str) -> String {
        match self {
            CodeLine::Code { code } => code.to_string(),

            CodeLine::Eval {
                code,
                marker,
                comment,
                ..
            } => {
                let mut out = format!("{code} {marker} {result}");
                if let Some(c) = comment {
                    out.push(' ');
                    out.push_str(c.trim());
                }
                out
            }

            CodeLine::EvalAssignment {
                code,
                marker,
                comment,
                ..
            } => {
                let mut out = format!("{code} {marker} {result}");
                if let Some(c) = comment {
                    out.push(' ');
                    out.push_str(c.trim());
                }
                out
            }
        }
    }
}

/// Split a Python line into structured form.
/// - `input`: line to parse
/// - `marker`: eval marker (e.g. "#=")
/// - `comment`: comment symbol (e.g. "#")
/// - `extract_assignment`: function that returns Some(var_name) if line is assignment
pub fn split_line<'a, F>(
    input: &'a str,
    marker: &'a str,
    comment: &str,
    extract_assignment: F,
) -> CodeLine<'a>
where
    F: Fn(&'a str) -> Option<&'a str>,
{
    let trimmed = input.trim();

    // 🟢 Case 1: eval line (contains marker)
    if let Some(marker_pos) = trimmed.find(marker) {
        let (before_marker, after_marker) = trimmed.split_at(marker_pos);
        let after_marker = &after_marker[marker.len()..];

        // Split into result and trailing comment
        let (result_part, comment_part) = if let Some(cpos) = after_marker.find(comment) {
            let (res, com) = after_marker.split_at(cpos);
            (res.trim(), Some(com))
        } else {
            (after_marker.trim(), None)
        };

        let result = if result_part.is_empty() {
            None
        } else {
            Some(result_part)
        };

        if let Some(var) = extract_assignment(before_marker.trim()) {
            CodeLine::EvalAssignment {
                var,
                code: before_marker.trim(),
                marker,
                result,
                comment: comment_part.map(|s| s.trim()),
            }
        } else {
            CodeLine::Eval {
                code: before_marker.trim(),
                marker,
                result,
                comment: comment_part.map(|s| s.trim()),
            }
        }
    } else {
        CodeLine::Code { code: trimmed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Very simple assignment detector for tests.
    /// Returns the variable name before '=' if one exists.
    /// Ignores spacing and does no syntax validation.
    pub fn extract_assignment_var(code: &str) -> Option<&str> {
        if let Some(eq_pos) = code.find('=') {
            let (left, right) = code.split_at(eq_pos);
            let var = left.trim();
            let rhs = right.trim().trim_start_matches('=').trim();

            if !var.is_empty() && !rhs.is_empty() {
                return Some(var);
            }
        }
        None
    }

    #[test]
    fn test_normal_code() {
        let line = "a = 1 + b # comment";
        let result = split_line(line, "#=", "#", extract_assignment_var);

        assert_eq!(
            result,
            CodeLine::Code {
                code: "a = 1 + b # comment"
            }
        );
    }

    #[test]
    fn test_eval() {
        let line = "b + 2 #= 6 # comment";
        let result = split_line(line, "#=", "#", extract_assignment_var);

        assert_eq!(
            result,
            CodeLine::Eval {
                code: "b + 2",
                marker: "#=",
                result: Some("6"),
                comment: Some("# comment"),
            }
        );
    }

    #[test]
    fn test_eval_no_result() {
        let line = "b + 2 #= # comment";
        let result = split_line(line, "#=", "#", extract_assignment_var);

        assert_eq!(
            result,
            CodeLine::Eval {
                code: "b + 2",
                marker: "#=",
                result: None,
                comment: Some("# comment"),
            }
        );
    }

    #[test]
    fn test_eval_assignment() {
        let line = "c = b + 2 #= 6 # comment";
        let result = split_line(line, "#=", "#", extract_assignment_var);

        assert_eq!(
            result,
            CodeLine::EvalAssignment {
                var: "c",
                code: "c = b + 2",
                marker: "#=",
                result: Some("6"),
                comment: Some("# comment"),
            }
        );
    }

    #[test]
    fn test_eval_assignment_no_comment() {
        let line = "x = y + 3 #= 10";
        let result = split_line(line, "#=", "#", extract_assignment_var);

        assert_eq!(
            result,
            CodeLine::EvalAssignment {
                var: "x",
                code: "x = y + 3",
                marker: "#=",
                result: Some("10"),
                comment: None,
            }
        );
    }

    #[test]
    fn test_eval_with_spaces() {
        let line = "  d + 4   #=    12    # some note   ";
        let result = split_line(line, "#=", "#", extract_assignment_var);

        assert_eq!(
            result,
            CodeLine::Eval {
                code: "d + 4",
                marker: "#=",
                result: Some("12"),
                comment: Some("# some note"),
            }
        );
    }

    #[test]
    fn test_empty_line() {
        let line = "";
        let result = split_line(line, "#=", "#", extract_assignment_var);
        assert_eq!(result, CodeLine::Code { code: "" });
    }
}
