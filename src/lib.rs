#[macro_use] extern crate error_chain;
#[macro_use] extern crate nom;
#[cfg(test)]
#[macro_use] extern crate nom_test_helpers;

use std::io::{self, Write};

use errors::*;

mod errors;
mod parse; 
mod cli;

pub struct Config {
    pub prompt: String,
}

pub fn run(config: &Config) -> Result<()> {
    loop {
        write!(&mut io::stdout(), "{}", config.prompt).chain_err(|| "Couldn't write prompt")?;
        io::stdout().flush().chain_err(|| "Couln't flush stdout")?;
        let mut inp = String::new();
        io::stdin().read_line(&mut inp).chain_err(|| "Couldn't read input")?;
        let inp = parse::parse_line(inp.trim().as_bytes());
        print!("{:?}", inp);
    }
}
