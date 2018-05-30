use nom;
use std::str;

use errors::*;
use {Buffer, Config};
use commands::Command;

fn lowercase() -> String {
    (97u8..123).map(|b| b as char)
               .map(|c| c.to_string())
               .collect::<Vec<_>>()
               .join("")
}

#[derive(Debug, PartialEq, Clone)]
pub struct LineRange(Option<Addr>, Mode, Option<Addr>);

impl LineRange {
    pub fn everything() -> LineRange {
        LineRange(Some(Addr::number(1)), Mode::Comma, Some(Addr::dollar_sign()))
    }

    pub fn current_line() -> LineRange {
        LineRange(Some(Addr::period()), Mode::Comma, Some(Addr::period()))
    }

    pub fn resolve(self, buffer: &Buffer, config: &Config) -> Result<(usize, usize)> {
        Ok((self.0.unwrap_or(Addr::number(1)).resolve(buffer, config)?,
            self.2.unwrap_or(Addr::dollar_sign()).resolve(buffer, config)?))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LineAddr {
    Number(u64),
    DollarSign,
    Period,
    Mark(char),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Modifier {
    PrefixPlus,
    SuffixPlus(Option<u64>),
    PrefixMinus,
    SuffixMinus(Option<u64>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Addr {
    pub primary: LineAddr,
    modifier: Option<Modifier>,
}

impl Addr {
    pub fn new(line_addr: LineAddr, modifier: Option<Modifier>) -> Addr {
        Addr {
            primary: line_addr,
            modifier: modifier,
        }
    }

    pub fn resolve(self, buffer: &Buffer, config: &Config) -> Result<usize> {
        Ok(match self.primary {
            LineAddr::Number(n) => {
                if n > 0 {
                    (n - 1) as usize
                } else {
                    0
                }
            },
            LineAddr::DollarSign => {
                if buffer.len() > 1 {
                    buffer.len() - 1
                } else {
                    0
                }
            },
            LineAddr::Period => config.current_index.unwrap_or(0usize) as usize,
            LineAddr::Mark(s) => match config.marks.get(&s) {
                Some(u) => *u,
                None => return Err("Mark not found".into()), 
            },
        })
    }

    pub fn number(num: u64) -> Addr {
        Addr::new(LineAddr::Number(num), None)
    }

    pub fn dollar_sign() -> Addr {
        Addr::new(LineAddr::DollarSign, None)
    }

    pub fn period() -> Addr {
        Addr::new(LineAddr::Period, None)
    }

    pub fn mark(c: char) -> Addr {
        Addr::new(LineAddr::Mark(c), None)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Mode {
    Comma,
    Semicolon,
}

/* Line Addressing */

named!(comma_or_semicolon< &str, Mode >, alt!(
                  tag!(",") => {|_| Mode::Comma}
                | tag!(";") => {|_| Mode::Semicolon}
));

named!(num_str< &str, u64 >,
            map_res!(is_a_s!("0123456789"), str::parse)
);

named!(number<&str, LineAddr>,
        do_parse!(
            num: num_str >>
            (LineAddr::Number(num))));

named!(dollar_sign<&str, LineAddr>,
        do_parse!(
            tag!("$") >>
            (LineAddr::DollarSign)));

named!(period<&str, LineAddr>,
        do_parse!(
            tag!(".") >>
            (LineAddr::Period)));

named!(mark<&str, LineAddr>,
        do_parse!(
            tag!("'") >>
            mark: one_of!(&lowercase()) >>
            (LineAddr::Mark(mark))
));

named!(line_addr<&str, LineAddr>, alt!(
              period
            | dollar_sign
            | number
            | mark
));

named!(prefix_plus_minus<&str, Modifier>,
        alt!(
              tag!("+") => {|_| Modifier::PrefixPlus }
            | tag!("-") => {|_| Modifier::PrefixMinus }
        )
);

named!(addr_prefix<&str, Addr>, 
        do_parse!(
            prefix: opt!(prefix_plus_minus) >>
            primary: call!(line_addr) >>
            (Addr::new(primary, prefix))
));

/*
named!(suffix_plus_minus<&str, Modifier>,
        alt!(
              tag!("+") => {|_| Modifier::SuffixPlus(None) }
            | tag!("-") => {|_| Modifier::SuffixMinus(None) }
        )
);

named!(addr_suffix<&str, Addr>,
        do_parse!(
            primary: call!(line_addr) >>
            suffix: opt!(suffix_plus_minus) >>
            (Addr::new(primary, suffix))
));
*/

named!(addr<&str, Addr>, call!(addr_prefix));

named!(range< &str, LineRange >, do_parse!(
            start: opt!(addr) >>
            mode: comma_or_semicolon >>
            end: opt!(addr) >>
            (LineRange(start, mode, end))
));

/* End Line Addressing */

/* Commands */

named!(print_lines< &str, Command >,
        do_parse!(
            range: opt!(range) >>
            tag!("p") >>
            (Command::Print(range))
));

named!(print_numbered_lines< &str, Command >,
        do_parse!(
            range: opt!(range) >>
            tag!("n") >>
            (Command::PrintNumbered(range))
));

named!(quit<&str, Command>,
        do_parse!(
            tag!("q") >>
            (Command::Quit)));

named!(hard_quit<&str, Command>,
        do_parse!(
            tag!("Q") >>
            (Command::HardQuit)
));

named!(toggle_show_prompt<&str, Command>,
        do_parse!(
            tag!("P") >>
            (Command::ToggleShowPrompt)
));

named!(delete<&str, Command>,
        do_parse!(
            range: opt!(range) >>
            tag!("d") >>
            (Command::Delete(range))
));

named!(append_text<&str, Command>,
        do_parse!(
            addr: opt!(addr) >>
            tag!("a") >>
            (Command::AppendText(addr))
));

named!(save_file<&str, Command>,
        do_parse!(
                range: opt!(range) >>
                filename: alt_complete!(
                      separated_pair!(
                          tag!("w"), call!(nom::multispace), call!(nom::rest_s)) => {|r: (_, &str)| 
                              Some(r.1.into())
                          }
                    | tag!("w") => {|_| None}
                ) >>
                (Command::SaveFile(range, filename)))
);

named!(save_append<&str, Command>,
        do_parse!(
            range: opt!(range) >>
            tag!("W") >>
            call!(nom::multispace) >>
            filename: opt!(call!(nom::rest_s)) >>
            (Command::SaveAppend(range, filename.map(|s| s.into())))
));

named!(save_and_quit<&str, Command>,
        do_parse!(
            range: opt!(range) >>
            tag!("wq") >>
            call!(nom::multispace) >>
            filename: opt!(call!(nom::rest_s)) >>
            (Command::SaveAndQuit(range, filename.map(|s| s.into())))
));

named!(default_filename<&str, Command>,
        alt_complete!(
              separated_pair!(
                  tag!("f"), call!(nom::multispace), call!(nom::rest_s)) => {|r: (&str, &str)|
                      Command::SetDefaultFilename(r.1.into())
                  }
            | tag!("f") => {|_| Command::GetDefaultFilename}
        )
);

named!(edit_file<&str, Command>,
        alt_complete!(
              separated_pair!(
                  tag!("e"), call!(nom::multispace), call!(nom::rest_s)) => {|r: (&str, &str)|
                      Command::EditFile(Some(r.1.into()))
                  }
            | tag!("e") => {|_| Command::EditFile(None)}
        )
);

named!(uncond_edit_file<&str, Command>,
        alt_complete!(
              separated_pair!(
                  tag!("E"), call!(nom::multispace), call!(nom::rest_s)) => {|r: (&str, &str)|
                      Command::UncondEditFile(Some(r.1.into()))
                  }
            | tag!("E") => {|_| Command::UncondEditFile(None)}
        )
);
        

named!(mark_line<&str, Command>,
        do_parse!(
            addr: opt!(addr) >>
            tag!("k") >>
            mark: one_of!(&lowercase()) >>
            (Command::MarkLine(addr, mark))
));

named!(insert_text<&str, Command>,
        do_parse!(
            addr: opt!(addr) >>
            tag!("i") >>
            (Command::InsertText(addr))
));

named!(change_text<&str, Command>,
        do_parse!(
            range: opt!(range) >>
            tag!("c") >>
            (Command::ChangeText(range))
));

named!(last_error<&str, Command>,
        do_parse!(
            tag!("h") >>
            (Command::LastError)
));

named!(toggle_error_expl<&str, Command>,
        do_parse!(
            tag!("H") >>
            (Command::ToggleErrorExpl)
));

named!(pub parse_line< &str, Command >,
        alt!(
              print_lines
            | print_numbered_lines
            | quit
            | hard_quit
            | toggle_show_prompt
            | append_text
            | delete
            | save_file
            | save_append
            | save_and_quit
            | default_filename
            | edit_file
            | uncond_edit_file
            | mark_line
            | insert_text
            | change_text
            | last_error
            | toggle_error_expl
        )
);

#[cfg(test)]
mod tests {
}
