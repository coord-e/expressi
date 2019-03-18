use ansi_term::Colour::Red;
use structopt::StructOpt;

use expressi::cli::{self, opts};

#[derive(StructOpt)]
enum Opt {
    #[structopt(name = "run")]
    Run {
        #[structopt(flatten)]
        opt: opts::RunOpt,
    },
    #[structopt(name = "build")]
    Build {
        #[structopt(flatten)]
        opt: opts::BuildOpt,
    },
}

#[cfg_attr(tarpaulin, skip)]
fn main() {
    let result = match Opt::from_args() {
        Opt::Run { opt } => cli::run(&opt).map(|_| ()),
        Opt::Build { opt } => cli::build(&opt),
    };
    if let Err(e) = result {
        eprintln!("{}: {}", Red.paint("Fatal Error"), e);
    }
}
