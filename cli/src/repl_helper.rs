use std::{collections::HashMap, io::Cursor};
use rustyline::{completion::Completer, hint::Hinter, highlight::Highlighter, validate::{Validator, MatchingBracketValidator}, Helper};
use syntect::{parsing::{SyntaxDefinition, SyntaxSet, SyntaxSetBuilder}, highlighting::{ThemeSet, Theme}, easy::HighlightLines, util::LinesWithEndings};
use roost::interpreter::value::Value;

pub struct ReplHelper {
    global_scope: HashMap<String, Value>,
    brackets: MatchingBracketValidator,
    syntaxes: SyntaxSet,
    theme: Theme,
}

impl ReplHelper {
    pub fn new(global_scope: HashMap<String, Value>) -> Self {
        return Self {
            global_scope,
            brackets: MatchingBracketValidator::new(),
            syntaxes: {
                let mut builder = SyntaxSetBuilder::new();
                builder.add(SyntaxDefinition::load_from_str(include_str!("res/roost.sublime-syntax"), true, None).unwrap());
                builder.build()
            },
            theme: ThemeSet::load_from_reader(&mut Cursor::new(include_str!("res/one-dark.tmTheme"))).unwrap(),
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
        if line.len() > pos { return None; }
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
    fn highlight<'l>(&self, line: &'l str, _: usize) -> std::borrow::Cow<'l, str> {
        let mut h = HighlightLines::new(self.syntaxes.find_syntax_by_name("roost").unwrap(), &self.theme);
        let mut out = String::new();
        for line in LinesWithEndings::from(line) {
            let ranges = h.highlight(line, &self.syntaxes);
            let escaped = syntect::util::as_24_bit_terminal_escaped(&ranges[..], false);
            out += &escaped;
        }
        return std::borrow::Cow::Owned(out);
    }

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

    fn highlight_char(&self, _: &str, _: usize) -> bool {
        return true;
    }
}

impl Validator for ReplHelper {
    fn validate(&self, ctx: &mut rustyline::validate::ValidationContext) -> rustyline::Result<rustyline::validate::ValidationResult> {
        return self.brackets.validate(ctx);
    }
}
