#[macro_use] extern crate error_chain;
#[macro_use] extern crate nom;
#[cfg(test)]
#[macro_use] extern crate nom_test_helpers;

use std::io::{self, Write};
use std::default::Default;

use errors::*;

mod errors;
mod parse; 
mod cli;
mod commands;

pub struct Config {
    pub prompt: String,
    pub dirty: bool,
    pub show_prompt: bool,
    pub current_index: usize,
    pub default_filename: Option<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            prompt: "* ".into(),
            dirty: false,
            show_prompt: false,
            current_index: 0,
            default_filename: None,
        }
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
        io::stdout().flush().chain_err(|| "Couln't flush stdout")?;
        let mut inp = String::new();
        io::stdin().read_line(&mut inp).chain_err(|| "Couldn't read input")?;
        let inp = parse::parse_line(inp.trim());
        let inp = match inp {
            nom::IResult::Done(_, o) => o,
            _ => continue,
        };
        println!("Command: {:?}", &inp);
        if let Err(n) = inp.run(&mut buffer, config) {
            ::std::process::exit(n as i32);
        };
    }
}
