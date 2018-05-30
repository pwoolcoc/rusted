extern crate env_logger;
#[macro_use]
extern crate structopt;
extern crate _rusted as rusted;

use std::path::PathBuf;
use structopt::StructOpt;
use rusted::Config;

#[derive(StructOpt, Debug)]
#[structopt(name = "rusted")]
struct Opts {
    #[structopt(short = "p", long = "prompt")]
    pub prompt: Option<String>,
    #[structopt(parse(from_os_str))]
    pub file: Option<PathBuf>,
}

fn main() {
    env_logger::init().unwrap();
    let opts = Opts::from_args();
    let mut config = Config::default();
    if let Some(p) = opts.prompt {
        config.prompt = p.to_string();
        config.show_prompt = true;
    }
    if let Some(f) = opts.file {
        config.default_filename = Some(f);
    }
    if let Err(_) = rusted::run(&mut config) {
        ::std::process::exit(1);
    }
}

