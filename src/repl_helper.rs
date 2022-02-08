use std::collections::HashMap;
use rustyline::{completion::Completer, hint::Hinter, highlight::Highlighter, validate::{Validator, MatchingBracketValidator}, Helper};
use crate::interpreter::value::Value;

pub struct ReplHelper {
    global_scope: HashMap<String, Value>,
    brackets: MatchingBracketValidator,
}

impl ReplHelper {
    pub fn new(global_scope: HashMap<String, Value>) -> Self {
        return Self {
            global_scope,
            brackets: MatchingBracketValidator::new(),
        }
    }
}

impl Helper for ReplHelper {}

impl Completer for ReplHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let mut name = String::new();
        let mut name_pos = pos;
        while let Some(char) = line.chars().nth(name_pos.wrapping_sub(1)) {
            if !char.is_ascii_alphanumeric() && char != '_' { break; }
            name.push(char);
            name_pos -= 1;
        }
        if name.is_empty() { return Ok((0, vec![])) }
        name = name.chars().rev().collect();

        let mut completions = vec![
            String::from("var"),
            String::from("true"),
            String::from("false"),
            String::from("null"),
            String::from("if"),
            String::from("else"),
            String::from("fun"),
            String::from("loop"),
            String::from("while"),
            String::from("for"),
            String::from("in"),
            String::from("return"),
            String::from("break"),
            String::from("continue"),

            String::from("print"),
            String::from("printl"),
            String::from("typeOf"),
            String::from("exit"),
            String::from("answer"),
        ];
        for name in self.global_scope.keys() {
            completions.push(name.clone());
        }
        completions = completions.iter().filter_map(|it| if it.starts_with(&name) { Some(it.clone()) } else { None }).collect();

        return Ok((
            name_pos,
            completions,
        ));
    }
}

impl Hinter for ReplHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        if let Ok((mut completion_pos, completions)) = self.complete(line, pos, ctx) {
            if completions.is_empty() { return None; }
            let mut hint = completions[0].clone();
            while completion_pos < pos {
                if hint.is_empty() { return None; }
                hint.remove(0);
                completion_pos += 1;
            }
            return Some(hint);
        } else { return None; };
    }
}

impl Highlighter for ReplHelper {
    // fn highlight<'l>(&self, line: &'l str, pos: usize) -> std::borrow::Cow<'l, str> {
    //     let _ = pos;
    //     std::borrow::Cow::Borrowed(line)
    // }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _: bool,
    ) -> std::borrow::Cow<'b, str> {
        std::borrow::Cow::Owned(format!("\x1b[1;32m{}\x1b[0m", prompt))
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        std::borrow::Cow::Owned(format!("\x1b[90m{}\x1b[0m", hint))
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _: rustyline::CompletionType,
    ) -> std::borrow::Cow<'c, str> {
        std::borrow::Cow::Owned(format!("\x1b[36m{}\x1b[0m", candidate))
    }

    // fn highlight_char(&self, line: &str, pos: usize) -> bool {
    //     let _ = (line, pos);
    //     false
    // }
}

impl Validator for ReplHelper {
    fn validate(&self, ctx: &mut rustyline::validate::ValidationContext) -> rustyline::Result<rustyline::validate::ValidationResult> {
        return self.brackets.validate(ctx);
    }
}
