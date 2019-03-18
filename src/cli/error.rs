use failure::Fail;

use std::io;
use std::path::PathBuf;

#[derive(Fail, Debug)]
pub enum CLIError {
    #[fail(display = "IO Error: {}", error)]
    IOError { error: io::Error },

    #[fail(display = "File not found: {:?}", path)]
    NotFound { path: PathBuf },
}
