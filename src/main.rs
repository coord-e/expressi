use ansi_term::Colour::Red;
use structopt::StructOpt;

use expressi::compile::build;
use expressi::compile::jit;
use expressi::compile::opts::{BuildOpt, RunOpt};

#[derive(StructOpt)]
enum Opt {
    #[structopt(name = "run")]
    Run {
        #[structopt(flatten)]
        opt: RunOpt,
    },
    #[structopt(name = "build")]
    Build {
        #[structopt(flatten)]
        opt: BuildOpt,
    },
}

#[cfg_attr(tarpaulin, skip)]
fn main() {
    let result = match Opt::from_args() {
        Opt::Run { opt } => jit::run(&opt).map(|_| ()),
        Opt::Build { opt } => build::build(opt),
    };
    if let Err(e) = result {
        eprintln!("{}: {}", Red.paint("Fatal Error"), e);
    }
}
