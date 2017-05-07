use std::str;

#[derive(Debug, PartialEq, Clone)]
pub struct LineRange(Option<u64>, Mode, Option<u64>);

#[derive(Debug, PartialEq, Clone)]
pub enum Mode {
    Comma,
    Semicolon,
}

named!(comma_or_semicolon< &[u8], Mode >, alt!(
                  tag!(",") => {|_| Mode::Comma}
                | tag!(";") => {|_| Mode::Semicolon}
));

named!(num_str< &[u8], &str >,
        map_res!(
            is_a!("0123456789"),
            str::from_utf8));

named!(range< &[u8], LineRange >, do_parse!(
            start: opt!(map_res!(num_str, str::parse)) >>
            mode: comma_or_semicolon >>
            end: opt!(map_res!(num_str, str::parse)) >>
            (LineRange(start, mode, end))
));

named!(pub parse_line< &[u8], LineRange >,
        call!(range));

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
