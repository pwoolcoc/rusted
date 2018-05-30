use parse::Addr;
use commands::{insert_all, unknown};
use {Buffer, Config};
use errors::*;

pub fn cmd(text: &[String], line: Option<Addr>,
           buffer: &mut Buffer, cfg: &mut Config) -> Result<()> {
    let (position, next_cur) = if buffer.is_empty() {
        cfg.current_index = Some(0);
        (0, text.len() - 1)
    } else {
        let position = line.unwrap_or(Addr::period())
                            .resolve(buffer, cfg)?;
        cfg.current_index = Some(position);
        (position + 1, text.len())
    };
    let num_lines = text.len();
    if num_lines == 0 {
        return Err(unknown());
    }
    let _ = insert_all(buffer, position, &text);
    cfg.update_curidx(next_cur);
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
        assert_eq!(config.current_index, Some(2));
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
        assert_eq!(config.current_index, Some(2));
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
        config.current_index = Some(0);
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
        assert_eq!(config.current_index, Some(3));
    }

    #[test]
    fn empty_input_text_changes_current_line_to_addressed_line() {
        let text = vec![];
        let mut buffer = vec![
            "the quick brown fox".into(),
            "jumps over the lazy dog".into(),
            "lorem ipsum".into(),
        ];
        let mut config = Config::default();
        config.current_index = Some(2);
        let addr = Some(Addr::number(2));
        let _ = cmd(&text, addr, &mut buffer, &mut config);
        assert_eq!(&buffer, &buffer); // buffer is unchanged
        assert_eq!(config.current_index, Some(1));
    }
}
