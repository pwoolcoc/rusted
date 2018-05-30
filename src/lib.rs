#[macro_use] extern crate error_chain;
#[macro_use] extern crate nom;
#[macro_use] extern crate log;
#[cfg(test)]
#[macro_use] extern crate nom_test_helpers;

use std::io::{self, Write};
use std::default::Default;
use std::collections::HashMap;
use std::process;
use std::path::PathBuf;

use errors::*;

const DEFAULT_PROMPT: &'static str = "*";

pub struct Config {
    pub prompt: String,
    pub dirty: bool,
    pub show_prompt: bool,
    pub current_index: Option<usize>,
    pub default_filename: Option<PathBuf>,
    pub cut_buffer: Vec<String>,
    pub marks: HashMap<char, usize>,
    pub last_error: Option<String>,
    pub print_errors: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            prompt: DEFAULT_PROMPT.into(),
            dirty: false,
            show_prompt: false,
            current_index: None,
            default_filename: None,
            cut_buffer: vec![],
            marks: HashMap::new(),
            last_error: None,
            print_errors: false,
        }
    }
}

impl Config {
    pub fn update_curidx(&mut self, idx: usize) {
        let loc = if let Some(v) = self.current_index.as_ref() {
            *v + idx
        } else {
            idx
        };
        self.current_index = Some(loc);
    }
}

pub type Buffer = Vec<String>;

pub fn insert_all(buffer: &mut Buffer, index: usize, elements: &[String]) -> Result<()> {
    for (idx, elem) in elements.into_iter().enumerate() {
        buffer.insert(index + idx, elem.to_owned());
    }
    Ok(())
}

pub fn run(config: &mut Config) -> Result<()> {
    let mut buffer = Buffer::new();
    loop {
        if config.show_prompt {
            write!(&mut io::stdout(), "{}", config.prompt)
                                .chain_err(|| "Couldn't write prompt")?;
        }
        io::stdout().flush().chain_err(|| "Couldn't flush stdout")?;
        let mut inp = String::new();
        io::stdin().read_line(&mut inp).chain_err(|| "Couldn't read input")?;
        let inp = parse::parse_line(inp.trim());
        let inp = match inp {
            nom::IResult::Done(_, o) => o,
            x => {
                debug!("Not done, got {:?}", x);
                continue;
            },
        };
        debug!("Command: {:?}, current index: {:?}", &inp, config.current_index);
        match inp.run(&mut buffer, config) {
            Err(Error(ErrorKind::Unknown, _)) => {
                println!("?");
            },
            Err(Error(ErrorKind::Msg(s), _)) => {
                if config.print_errors {
                    println!("{}", s);
                } else {
                    println!("?");
                }
                config.last_error = Some(s);
            },
            Err(_) => process::exit(1),
            _ => (),
        };
    }
}

mod errors;
mod parse;
mod cli;
mod commands;
