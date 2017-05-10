use parse::{LineRange, LineAddr};
use {Buffer, Config, insert_all};

use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{self, Write, BufRead, BufReader};

#[allow(dead_code)] // take this out when all the "TODO"s are gone
#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    AppendText(Option<LineAddr>),
    ChangeText(Option<LineRange>),                      // TODO
    Delete(Option<LineRange>),
    EditFile(Option<String>),
    UncondEditFile(Option<String>),                     // TODO
    SetDefaultFilename(String),
    GetDefaultFilename,
    Global(Option<LineRange>, String, String),          // TODO
    InteractiveGlobal(Option<LineRange>, String),       // TODO
    LastError,                                          // TODO
    ToggleErrorExpl,                                    // TODO
    InsertText(Option<LineAddr>),                       // TODO
    JoinLines(Option<LineRange>),                       // TODO
    MarkLine(Option<LineAddr>, String),                 // TODO
    List(Option<LineRange>),                            // TODO
    MoveLines(Option<LineRange>, Option<LineAddr>),     // TODO
    PrintNumbered(Option<LineRange>),
    Print(Option<LineRange>),
    ToggleShowPrompt,
    Quit,
    HardQuit,
    ReadFile(Option<LineAddr>, String),                 // TODO
    Substitute(Option<LineRange>, String, String),      // TODO
    RepeatSubst(Option<LineRange>),                     // TODO
    Transfer(Option<LineRange>, Option<LineAddr>),      // TODO
    Undo,                                               // TODO
    NotGlobal(Option<LineRange>, String, String),       // TODO
    InteractiveNotGlobal(Option<LineRange>, String),    // TODO
    SaveFile(Option<LineRange>, Option<String>),
    SaveAndQuit(Option<LineRange>, Option<String>),
    SaveAppend(Option<LineRange>, Option<String>),

    InsertFromCut(Option<LineAddr>),                    // TODO
    YankToCut(Option<LineRange>),                       // TODO
    Scroll(Option<LineAddr>),                           // TODO
    ShellCmd(String),                                   // TODO
    Comment(Option<LineRange>),                         // TODO
    PrintLineNumber(Option<LineAddr>),                  // TODO
    NullCmd(Option<LineAddr>),                          // TODO
}

pub enum CommandResult {
    Success,
    Err(u8),
    Unknown,
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

fn get_filename(filename: Option<String>, cfg: &mut Config) -> Option<String> {
    let no_default = cfg.default_filename.is_none();
    match filename {
        Some(f) => {
            if no_default {
                cfg.default_filename = Some(f.to_owned());
            }
            Some(f)
        },
        None => {
            if no_default {
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
                cfg: &mut Config) -> CommandResult
{
    let filename = match get_filename(filename, cfg) {
        Some(f) => f,
        None => return CommandResult::Err(1),
    };

    if filename.trim().starts_with("!") {
        // system command
        return CommandResult::Unknown;
    }

    let path = Path::new(&filename);
    if !path.exists() {
        return CommandResult::Unknown;
    }
    let mut fp = match open_options.open(&path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("Could not open file");
            return CommandResult::Err(255);
        }
    };
    for idx in start..end {
        let _ = writeln!(fp, "{}", buffer[idx]);
    }
    cfg.dirty = false;
    CommandResult::Success
}

fn confirm(msg: &str) -> bool {
    let _ = write!(&mut io::stdout(), "{} (y/N) ", msg);
    let _ = io::stdout().flush();
    let mut inp = String::new();
    let _ = io::stdin().read_line(&mut inp);
    &inp.trim()[..] == "y"
}

fn quit(cfg: &mut Config) -> CommandResult {
    if cfg.dirty {
        if confirm("unsaved changes. really exit?") {
            CommandResult::Err(0)
        } else {
            CommandResult::Success
        }
    } else {
        CommandResult::Err(0)
    }
}

impl Command {
    pub fn run(self, buffer: &mut Buffer, cfg: &mut Config) -> CommandResult {
        match self {
            Command::ToggleShowPrompt => {
                cfg.show_prompt = !cfg.show_prompt;
                CommandResult::Success
            },
            Command::SetDefaultFilename(filename) => {
                cfg.default_filename = Some(filename.trim().into());
                CommandResult::Success
            },
            Command::GetDefaultFilename => {
                match cfg.default_filename {
                    Some(ref f) => {
                        println!("{}", f);
                        return CommandResult::Success;
                    },
                    None => return CommandResult::Unknown,
                }
            },
            Command::Print(range) => {
                if buffer.len() == 0 {
                    return CommandResult::Unknown;
                }

                let range = range.unwrap_or(LineRange::current_line())
                                 .resolve(buffer, cfg);
                let start = range.0;
                let end = range.1 + 1;
                let _ = writeln!(&mut io::stdout(), "{}",
                                buffer[start..end].join("\n"));
                let _ = io::stdout().flush();
                CommandResult::Success
            },
            Command::PrintNumbered(range) => {
                if buffer.len() == 0 {
                    return CommandResult::Unknown;
                }

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
                CommandResult::Success
            },
            Command::AppendText(line) => {
                let text = input_mode();
                let position = line.unwrap_or(LineAddr::Period)
                                   .resolve(buffer, cfg);
                let position = if buffer.is_empty() {
                    0
                } else {
                    position + 1
                };
                let _ = insert_all(buffer, position, &text);
                cfg.current_index += text.len() - 1;
                cfg.dirty = true;
                CommandResult::Success
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
                CommandResult::Success
            },
            Command::SaveFile(range, filename) => {
                let range = range.unwrap_or(LineRange::everything())
                                 .resolve(buffer, cfg);
                let start = range.0;
                let end = range.1 + 1;
                let mut oo = OpenOptions::new();
                save_file(start, end, oo.truncate(true).write(true), filename, buffer, cfg)
            },
            Command::SaveAndQuit(range, filename) => {
                let range = range.unwrap_or(LineRange::everything())
                                 .resolve(buffer, cfg);
                let start = range.0;
                let end = range.1 + 1;
                let mut oo = OpenOptions::new();
                if let CommandResult::Err(e) = save_file(start, end,
                                                         oo.write(true),
                                                         filename, buffer,
                                                         cfg) {
                    return CommandResult::Err(e);
                }
                quit(cfg)
            },
            Command::SaveAppend(range, filename) => {
                let range = range.unwrap_or(LineRange::everything())
                                 .resolve(buffer, cfg);
                let start = range.0;
                let end = range.1 + 1;
                let mut oo = OpenOptions::new();
                save_file(start, end, oo.append(true), filename, buffer, cfg)
            },
            Command::EditFile(filename) => {
                let filename = match filename {
                    Some(filename) => {
                        filename
                    },
                    None => {
                        if cfg.default_filename.is_none() {
                            return CommandResult::Unknown;
                        } else {
                            cfg.default_filename.clone().unwrap()
                        }
                    }
                };
                if cfg.dirty && !confirm("unsaved changes. really edit?") {
                    return CommandResult::Success;
                }
                let path = Path::new(&filename);
                if !path.exists() {
                    return CommandResult::Unknown;
                }
                let fil = match File::open(&path) {
                    Ok(f) => f,
                    Err(_) => {
                        eprintln!("error opening file");
                        return CommandResult::Err(255);
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
                            eprintln!("error reading from file");
                            return CommandResult::Err(255);
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

                cfg.current_index = buffer.len() - 1;

                CommandResult::Success
            },
            Command::HardQuit => CommandResult::Err(0),
            Command::Quit => {
                quit(cfg)
            },
            _ => CommandResult::Success,
        }
    }
}
