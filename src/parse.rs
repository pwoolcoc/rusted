use nom;
use std::str;
use {Buffer, Config};
use commands::Command;

#[derive(Debug, PartialEq, Clone)]
pub struct LineRange(Option<LineAddr>, Mode, Option<LineAddr>);

impl LineRange {
    pub fn everything() -> LineRange {
        LineRange(Some(LineAddr::Number(1)), Mode::Comma, Some(LineAddr::DollarSign))
    }

    pub fn current_line() -> LineRange {
        LineRange(Some(LineAddr::Period), Mode::Comma, Some(LineAddr::Period))
    }

    pub fn resolve(self, buffer: &Buffer, config: &Config) -> (usize, usize) {
        (self.0.unwrap_or(LineAddr::Number(1)).resolve(buffer, config),
         self.2.unwrap_or(LineAddr::DollarSign).resolve(buffer, config))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum LineAddr {
    Number(u64),
    DollarSign,
    Period,
}

impl LineAddr {
    pub fn resolve(self, buffer: &Buffer, config: &Config) -> usize {
        match self {
            LineAddr::Number(n) => (n - 1) as usize,
            LineAddr::DollarSign => {
                buffer.len() - 1
            },
            LineAddr::Period => config.current_index as usize,
        }
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

named!(line_addr<&str, LineAddr>, alt!(
              period
            | dollar_sign
            | number
));

named!(range< &str, LineRange >, do_parse!(
            start: opt!(line_addr) >>
            mode: comma_or_semicolon >>
            end: opt!(line_addr) >>
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
            addr: opt!(line_addr) >>
            tag!("a") >>
            (Command::AppendText(addr))
));

named!(save_file<&str, Command>,
        do_parse!(
                range: opt!(range) >>
                tag!("w") >>
                call!(nom::multispace) >>
                filename: call!(nom::rest_s) >>
                (Command::SaveFile(range, filename.into()))
));

named!(save_and_quit<&str, Command>,
        do_parse!(
            range: opt!(range) >>
            tag!("wq") >>
            call!(nom::multispace) >>
            filename: call!(nom::rest_s) >>
            (Command::SaveAndQuit(range, filename.into()))
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
            | save_and_quit
        )
);

#[cfg(test)]
mod tests {
    use super::{range, LineRange, Mode};
    #[test]
    fn it_works() {
        assert_finished_and_eq!(
                range(b"23,24"),
                LineRange(Some(23), Mode::Comma, Some(24)));
    }
}
