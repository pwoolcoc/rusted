use parse::{LineRange, Addr};
use {Buffer, Config, insert_all};
use errors::*;

use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{self, Write, BufRead, BufReader};

mod append_text;
mod change_text;

#[allow(dead_code)] // take this out when all the "TODO"s are gone
#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    AppendText(Option<Addr>),
    ChangeText(Option<LineRange>),
    Delete(Option<LineRange>),
    EditFile(Option<String>),
    UncondEditFile(Option<PathBuf>),
    SetDefaultFilename(String),
    GetDefaultFilename,
    Global(Option<LineRange>, String, String),          // TODO
    InteractiveGlobal(Option<LineRange>, String),       // TODO
    LastError,
    ToggleErrorExpl,
    InsertText(Option<Addr>),
    JoinLines(Option<LineRange>),                       // TODO
    MarkLine(Option<Addr>, char),
    List(Option<LineRange>),                            // TODO
    MoveLines(Option<LineRange>, Option<Addr>),     // TODO
    PrintNumbered(Option<LineRange>),
    Print(Option<LineRange>),
    ToggleShowPrompt,
    Quit,
    HardQuit,
    ReadFile(Option<Addr>, String),                 // TODO
    Substitute(Option<LineRange>, String, String),      // TODO
    RepeatSubst(Option<LineRange>),                     // TODO
    Transfer(Option<LineRange>, Option<Addr>),      // TODO
    Undo,                                               // TODO
    NotGlobal(Option<LineRange>, String, String),       // TODO
    InteractiveNotGlobal(Option<LineRange>, String),    // TODO
    SaveFile(Option<LineRange>, Option<String>),
    SaveAndQuit(Option<LineRange>, Option<String>),
    SaveAppend(Option<LineRange>, Option<String>),

    InsertFromCut(Option<Addr>),                    // TODO
    YankToCut(Option<LineRange>),                       // TODO
    Scroll(Option<Addr>),                           // TODO
    ShellCmd(String),                                   // TODO
    Comment(Option<LineRange>),                         // TODO
    PrintLineNumber(Option<Addr>),                  // TODO
    NullCmd(Option<Addr>),                          // TODO
}

fn unknown() -> Error {
    ErrorKind::Unknown.into()
}

fn exit() -> Error {
    ErrorKind::Exit.into()
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

fn get_filename(filename: Option<String>, cfg: &mut Config) -> Option<PathBuf> {
    let no_default = cfg.default_filename.is_none();
    match filename {
        Some(f) => {
            let p: PathBuf = Path::new(&f).into();
            if no_default {
                debug!("no default filename, setting to {}", &f);
                cfg.default_filename = Some(p.clone());
            }
            Some(p)
        },
        None => {
            if no_default {
                debug!("no filename and no default filename :(");
                None
            } else {
                cfg.default_filename.clone()
            }
        }
    }
}

fn save_file(start: usize, end: usize,
                open_options: &mut OpenOptions,
                filename: Option<String>, buffer: &mut Buffer,
                cfg: &mut Config) -> Result<()>
{
    if let Some(ref f) = filename {
        // system command
        if f.trim().starts_with("!") {
            return Err(unknown());
        }
    }

    let filename = match get_filename(filename, cfg) {
        Some(f) => f,
        None => return Err("No filename".into()),
    };

    let path = Path::new(&filename);
    if !path.exists() {
        debug!("file does not exist");
        if confirm("file does not exist. create it?") {
            open_options.create(true);
        } else {
            return Err("file does not exist".into());
        }
    }
    let mut fp = match open_options.open(&path) {
        Ok(f) => f,
        Err(_) => {
            return Err("Could not open file".into());
        }
    };
    for idx in start..end {
        let _ = writeln!(fp, "{}", buffer[idx]);
    }
    cfg.dirty = false;
    Ok(())
}

fn confirm(msg: &str) -> bool {
    let _ = write!(&mut io::stdout(), "{} (y/N) ", msg);
    let _ = io::stdout().flush();
    let mut inp = String::new();
    let _ = io::stdin().read_line(&mut inp);
    &inp.trim()[..] == "y"
}

fn quit(cfg: &mut Config) -> Result<()> {
    if cfg.dirty {
        if confirm("unsaved changes. really exit?") {
            Err(exit())
        } else {
            Ok(())
        }
    } else {
        Err(exit())
    }
}

fn edit_file<P: AsRef<Path>>(filename: P, buffer: &mut Buffer, cfg: &mut Config) -> Result<()> {
    let path = filename.as_ref();
    if !path.exists() {
        return Err(unknown());
    }
    let fil = match File::open(&path) {
        Ok(f) => f,
        Err(_) => {
            return Err("error opening file".into());
        }
    };
    let reader = BufReader::new(fil);
    let lines = reader.lines();
    let size_hint = match lines.size_hint() {
        (_, Some(upper)) => upper,
        (lower, _) => lower,
    };
    let mut next_buffer = Vec::with_capacity(size_hint);
    for (idx, line) in lines.enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => {
                return Err("error reading from file".into());
            },
        };
        next_buffer.insert(idx, line);
    }

    if next_buffer.len() > buffer.len() {
        // reserve some more capacity for the buffer
        let extra = next_buffer.len() - buffer.len();
        buffer.reserve(extra);
    }

    buffer.clear();

    for (idx, elem) in next_buffer.into_iter().enumerate() {
        buffer.insert(idx, elem);
    }

    cfg.current_index = Some(buffer.len() - 1);

    Ok(())
}

impl Command {
    pub fn run(self, buffer: &mut Buffer, cfg: &mut Config) -> Result<()> {
        match self {
            Command::AppendText(line) => {
                let text = input_mode();
                append_text::cmd(&text, line, buffer, cfg)
            },
            Command::ChangeText(range) => {
                let text = input_mode();
                change_text::cmd(&text, range, buffer, cfg)
            },
            Command::Delete(range) => {
                let range = range.unwrap_or(LineRange::current_line())
                                 .resolve(buffer, cfg)?;
                let (start, end) = (range.0, range.1 + 1);
                for _ in start..end {
                    buffer.remove(start);
                }
                cfg.current_index = Some(start);
                cfg.dirty = true;
                Ok(())
            },
            Command::EditFile(filename) => {
                let filename = match filename {
                    Some(filename) => {
                        Path::new(&filename).into()
                    },
                    None => {
                        if cfg.default_filename.is_none() {
                            return Err(unknown());
                        } else {
                            cfg.default_filename.clone().unwrap()
                        }
                    }
                };
                if cfg.dirty && !confirm("unsaved changes. really edit?") {
                    return Ok(());
                }
                edit_file(&filename, buffer, cfg)
            },
            Command::UncondEditFile(filename) => {
                let filename = match filename {
                    Some(filename) => {
                        filename
                    },
                    None => {
                        if cfg.default_filename.is_none() {
                            return Err(unknown());
                        } else {
                            cfg.default_filename.clone().unwrap()
                        }
                    }
                };
                edit_file(&filename, buffer, cfg)
            },
            Command::SetDefaultFilename(filename) => {
                cfg.default_filename = Some(filename.trim().into());
                Ok(())
            },
            Command::GetDefaultFilename => {
                match cfg.default_filename {
                    Some(ref f) => {
                        debug!("default filename: {:?}", f);
                        return Ok(());
                    },
                    None => return Err(unknown()),
                }
            },
            Command::LastError => {
                if let Some(ref e) = cfg.last_error {
                    println!("{}", e);
                }
                Ok(())
            },
            Command::ToggleErrorExpl => {
                cfg.print_errors = !cfg.print_errors;
                Ok(())
            },
            Command::InsertText(line) => {
                let text = input_mode();
                let line = if buffer.is_empty() {
                    0
                } else {
                    line.unwrap_or(Addr::period()).resolve(buffer, cfg)?
                };
                let _ = insert_all(buffer, line, &text);
                cfg.update_curidx(text.len() - 1);
                cfg.dirty = true;
                Ok(())
            },
            Command::MarkLine(line, mark) => {
                let line = line.unwrap_or(Addr::period())
                               .resolve(buffer, cfg)?;
                debug!("Putting mark {} at line {}", mark, line);
                cfg.marks.insert(mark, line);
                Ok(())
            },
            Command::Print(range) => {
                if buffer.len() == 0 {
                    return Err(unknown());
                }

                let range = range.unwrap_or(LineRange::current_line())
                                 .resolve(buffer, cfg)?;
                let (start, end) = (range.0, range.1 + 1);
                let _ = writeln!(&mut io::stdout(), "{}",
                                buffer[start..end].join("\n"));
                let _ = io::stdout().flush();
                Ok(())
            },
            Command::PrintNumbered(range) => {
                if buffer.len() == 0 {
                    return Err(unknown());
                }

                let range = range.unwrap_or(LineRange::current_line())
                                 .resolve(buffer, cfg)?;
                let (start, end) = (range.0, range.1 + 1);
                let buf = buffer.iter().enumerate().map(|(ref idx, ref line)| {
                    format!("{}\t{}", idx + 1, line)
                }).collect::<Vec<_>>();
                let _ = writeln!(&mut io::stdout(), "{}",
                                buf[start..end].join("\n"));
                let _ = io::stdout().flush();
                Ok(())
            },
            Command::ToggleShowPrompt => {
                cfg.show_prompt = !cfg.show_prompt;
                Ok(())
            },
            Command::HardQuit => Err(exit()),
            Command::Quit => {
                quit(cfg)
            },
            Command::SaveFile(range, filename) => {
                let range = range.unwrap_or(LineRange::everything())
                                 .resolve(buffer, cfg)?;
                let start = range.0;
                let end = range.1 + 1;
                let mut oo = OpenOptions::new();
                save_file(start, end, oo.truncate(true).write(true), filename, buffer, cfg)
            },
            Command::SaveAndQuit(range, filename) => {
                let range = range.unwrap_or(LineRange::everything())
                                 .resolve(buffer, cfg)?;
                let start = range.0;
                let end = range.1 + 1;
                let mut oo = OpenOptions::new();
                save_file(start, end, oo.write(true), filename, buffer, cfg)?;
                quit(cfg)
            },
            Command::SaveAppend(range, filename) => {
                let range = range.unwrap_or(LineRange::everything())
                                 .resolve(buffer, cfg)?;
                let start = range.0;
                let end = range.1 + 1;
                let mut oo = OpenOptions::new();
                save_file(start, end, oo.append(true), filename, buffer, cfg)
            },
            _ => Ok(()),
        }
    }
}
