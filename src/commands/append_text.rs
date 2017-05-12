use parse::Addr;
use commands::{insert_all, unknown};
use {Buffer, Config};
use errors::*;

pub fn cmd(text: &[String], line: Option<Addr>,
           buffer: &mut Buffer, cfg: &mut Config) -> Result<()> {
    let position = line.unwrap_or(Addr::period())
                        .resolve(buffer, cfg)?;
    let position = if buffer.is_empty() {
        0
    } else {
        position + 1
    };
    let _ = insert_all(buffer, position, &text);
    let num_lines = text.len();
    if num_lines == 0 {
        return Err(unknown());
    }
    cfg.current_index += num_lines - 1;
    cfg.dirty = true;
    Ok(())
}

#[cfg(test)]
mod tests {
    use {Config};
    use parse::Addr;
    use super::cmd;

    #[test]
    fn default_line_empty_buffer() {
        let text = vec![
            "hello, world!".into(),
            "the quick brown fox".into(),
            "jumps over the lazy dog".into(),
        ];
        let mut buffer = vec![];
        let mut config = Config::default();
        let res = cmd(&text, None, &mut buffer, &mut config);
        assert!(res.is_ok());
        assert_eq!(&buffer, &text);
    }

    #[test]
    fn explicit_line_zero_empty_buffer() {
        let text = vec![
            "hello, world!".into(),
            "the quick brown fox".into(),
            "jumps over the lazy dog".into(),
        ];
        let mut buffer = vec![];
        let mut config = Config::default();
        let addr = Some(Addr::number(0));
        let res = cmd(&text, addr, &mut buffer, &mut config);
        assert!(res.is_ok());
        assert_eq!(&buffer, &text);
    }

    #[test]
    fn explicit_line_one_empty_buffer() {
        let text = vec![
            "hello, world!".into(),
            "the quick brown fox".into(),
            "jumps over the lazy dog".into(),
        ];
        let mut buffer = vec![];
        let mut config = Config::default();
        let addr = Some(Addr::number(1));
        let res = cmd(&text, addr, &mut buffer, &mut config);
        assert!(res.is_ok());
        assert_eq!(&buffer, &text);
    }

    #[test]
    fn explicit_line_populated_buffer() {
        let text = vec![
            "hello, world!".into(),
            "the quick brown fox".into(),
            "jumps over the lazy dog".into(),
        ];
        let mut buffer = vec![
            "this is already here".into(),
        ];
        let mut config = Config::default();
        let addr = Some(Addr::number(1));
        let res = cmd(&text, addr, &mut buffer, &mut config);
        let expected: Vec<String> = vec![
            "this is already here".into(),
            "hello, world!".into(),
            "the quick brown fox".into(),
            "jumps over the lazy dog".into(),
        ];
        assert!(res.is_ok());
        assert_eq!(&buffer, &expected);
    }
}
