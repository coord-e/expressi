use failure::Error;
use rustyline::Editor;

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
    history_file: String,
}

impl Shell {
    pub fn new(history_file: &str) -> Shell {
        let mut editor = Editor::<()>::new();
        if editor.load_history(history_file).is_err() {
            eprintln!("No previous history.");
        }
        Shell {
            line_count: 0,
            editor: editor,
            history_file: history_file.to_string(),
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
