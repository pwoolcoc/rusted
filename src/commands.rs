use parse::{LineRange, LineAddr};
use {Buffer, Config, insert_all};

use std::path::Path;
use std::fs::OpenOptions;
use std::io::{self, Write};

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Print(Option<LineRange>),
    PrintNumbered(Option<LineRange>),
    ToggleShowPrompt,
    AppendText(Option<LineAddr>),
    Delete(Option<LineRange>),
    SaveFile(Option<LineRange>, String),
    SaveAndQuit(Option<LineRange>, String),
    SaveAppend(Option<LineRange>, String),
    Quit,
    HardQuit,
}

pub fn input_mode() -> Vec<String> {
    let mut inp = vec![];
    loop {
        let mut s = String::new();
        let _ = io::stdin().read_line(&mut s);
        let s = s.trim();
        if &s[..] == "." {
            break;
        }
        inp.push(s.into());
    }
    inp
}

fn save_file(start: usize, end: usize,
               filename: &str, buffer: &mut Buffer,
                cfg: &mut Config) -> ::std::result::Result<(), u8>
{
    if filename.trim().starts_with("!") {
        // send to stdin of external command
    } else {
        let path = Path::new(&filename);
        if !path.exists() {
            println!("{}", &format!("NOT SAVED. File does not exist: {}", filename));
            return Ok(());
        }
        let mut fp = match OpenOptions::new().write(true).open(&path) {
            Ok(f) => f,
            Err(_) => {
                // todo stderr
                println!("Could not open file");
                return Err(255);
            }
        };
        for idx in start..end {
            let _ = writeln!(fp, "{}", buffer[idx]);
        }
    }
    cfg.dirty = false;
    Ok(())
}

fn quit(cfg: &mut Config) -> ::std::result::Result<(), u8> {
    if cfg.dirty {
        let _ = write!(&mut io::stdout(),
                "unsaved changes. really exit? (y/N) ");
        let _ = io::stdout().flush();
        let mut inp = String::new();
        let _ = io::stdin().read_line(&mut inp);
        if &inp.trim()[..] == "y" {
            Err(0)
        } else {
            Ok(())
        }
    } else {
        Err(0)
    }
}

impl Command {
    pub fn run(self, buffer: &mut Buffer, cfg: &mut Config)
            -> ::std::result::Result<(), u8>
    {
        match self {
            Command::ToggleShowPrompt => {
                cfg.show_prompt = !cfg.show_prompt;
                Ok(())
            },
            Command::Print(range) => {
                let range = range.unwrap_or(LineRange::current_line())
                                 .resolve(buffer, cfg);
                let start = range.0;
                let end = range.1 + 1;
                let _ = writeln!(&mut io::stdout(), "{}",
                                buffer[start..end].join("\n"));
                let _ = io::stdout().flush();
                Ok(())
            },
            Command::PrintNumbered(range) => {
                let range = range.unwrap_or(LineRange::current_line())
                                 .resolve(buffer, cfg);
                let start = range.0;
                let end = range.1 + 1;
                let buf = buffer.iter().enumerate().map(|(ref idx, ref line)| {
                    format!("{}\t{}", idx + 1, line)
                }).collect::<Vec<_>>();
                let _ = writeln!(&mut io::stdout(), "{}",
                                buf[start..end].join("\n"));
                let _ = io::stdout().flush();
                Ok(())
            },
            Command::AppendText(line) => {
                let text = input_mode();
                let position = line.unwrap_or(LineAddr::Period)
                                   .resolve(buffer, cfg);
                let _ = insert_all(buffer, position, &text);
                cfg.current_index += text.len() - 1;
                cfg.dirty = true;
                Ok(())
            },
            Command::Delete(range) => {
                let range = range.unwrap_or(LineRange::current_line())
                                 .resolve(buffer, cfg);
                let start = range.0;
                let end = range.1 + 1;
                for _ in start..end {
                    buffer.remove(start);
                }
                cfg.current_index = start;
                cfg.dirty = true;
                Ok(())
            },
            Command::SaveFile(range, filename) => {
                let range = range.unwrap_or(LineRange::everything())
                                 .resolve(buffer, cfg);
                let start = range.0;
                let end = range.1 + 1;
                save_file(start, end, &filename, buffer, cfg)
            },
            Command::SaveAndQuit(range, filename) => {
                let range = range.unwrap_or(LineRange::everything())
                                 .resolve(buffer, cfg);
                let start = range.0;
                let end = range.1 + 1;
                save_file(start, end, &filename, buffer, cfg).and_then(|_| {
                        quit(cfg)
                })
            },
            Command::SaveAppend(range, filename) => {
                let range = range.unwrap_or(LineRange::everything())
                                 .resolve(buffer, cfg);
                let start = range.0;
                let end = range.1 + 1;
                let path = Path::new(&filename);
                if !path.exists() {
                    println!("{}", &format!("NOT SAVED. File does not exist: {}", filename));
                    return Ok(());
                }
                let mut fp = match OpenOptions::new().append(true).open(&path) {
                    Ok(f) => f,
                    Err(_) => {
                        // todo stderr
                        println!("Could not open file");
                        return Err(255);
                    }
                };
                for idx in start..end {
                    let _ = writeln!(fp, "{}", buffer[idx]);
                }
                cfg.dirty = false;
                Ok(())
            },
            Command::HardQuit => Err(0),
            Command::Quit => {
                quit(cfg)
            },
            _ => Ok(()),
        }
    }
}
