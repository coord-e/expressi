use failure::Error;
use rustyline::Editor;

pub struct Shell {
    line_count: i32,
    editor: Editor<()>,
    history_file: String
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
            history_file: history_file.to_string()
        }
    }

    pub fn get_next_line(&mut self) -> Result<String, Error> {
        let prompt = format!("{}: > ", self.line_count);
        self.editor.readline(&prompt)
            .map(|line| {
                self.editor.add_history_entry(line.as_ref());
                self.line_count += 1;
                line
            })
            .map_err(Into::into)
    }

    pub fn save_history(&mut self) -> Result<(), Error> {
        self.editor.save_history(&self.history_file)
            .map_err(Into::into)
    }
}
