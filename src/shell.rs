use failure::Error;
use rustyline::Editor;

use std::path::{Path, PathBuf};

fn count_bracket_pair(buffer: &str) -> i32 {
    buffer
        .chars()
        .map(|c| match c {
            '(' | '{' | '[' => 1,
            ')' | '}' | ']' => -1,
            _ => 0,
        })
        .sum()
}

pub struct Shell {
    line_count: i32,
    editor: Editor<()>,
    history_file: PathBuf,
}

impl Shell {
    pub fn new<T: AsRef<Path>>(history_file: T) -> Shell {
        let mut editor = Editor::<()>::new();
        let _ = editor.load_history(&history_file); // ignore error

        Shell {
            line_count: 0,
            history_file: history_file.as_ref().into(),
            editor,
        }
    }

    fn get_next_single_line(&mut self, cont: &str, level: usize) -> Result<String, Error> {
        let prompt = format!("{1}: > {0} ", "..".repeat(level), self.line_count);
        let line = self.editor.readline(&prompt)?;
        let buffer = cont.to_string() + &line;
        let level = count_bracket_pair(&buffer);
        if level > 0 {
            self.get_next_single_line(&buffer, level as usize)
        } else {
            Ok(buffer)
        }
    }

    pub fn get_next_line(&mut self) -> Result<String, Error> {
        let buffer = self.get_next_single_line("", 0)?;
        self.editor.add_history_entry(buffer.as_ref());
        self.save_history()?;

        self.line_count += 1;
        Ok(buffer)
    }

    pub fn save_history(&mut self) -> Result<(), Error> {
        self.editor
            .save_history(&self.history_file)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::count_bracket_pair;

    #[test]
    fn bracket_pair_simple() {
        assert_eq!(count_bracket_pair("()"), 0);
    }

    #[test]
    fn bracket_pair_nothing() {
        assert_eq!(count_bracket_pair("string"), 0);
    }

    #[test]
    fn bracket_pair_opening() {
        assert_eq!(count_bracket_pair("(("), 2);
    }

    #[test]
    fn bracket_pair_closing() {
        assert_eq!(count_bracket_pair("))"), -2);
    }

    #[test]
    fn bracket_pair_mixed() {
        assert_eq!(count_bracket_pair("(a(a)"), 1);
    }

    #[test]
    fn bracket_pair_mixed_type() {
        assert_eq!(count_bracket_pair("(a[(a{a})"), 2);
    }
}
