use commands::{unknown, insert_all};
use {Buffer, Config};
use parse::LineRange;
use errors::*;

pub fn cmd(text: &[String], range: Option<LineRange>,
        buffer: &mut Buffer, cfg: &mut Config) -> Result<()> {
    let num_lines = text.len();
    if num_lines == 0 {
        return Err(unknown());
    }
    let range = range.unwrap_or(LineRange::current_line())
                        .resolve(buffer, cfg)?;
    let (start, end) = (range.0, range.1 + 1);
    if !buffer.is_empty() {
        for _ in start..end {
            buffer.remove(start);
        }
    }
    insert_all(buffer, start, &text)?;
    cfg.update_curidx(num_lines - 1);
    cfg.dirty = true;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::cmd;
    use {Config};
    use parse::LineRange;

    #[test]
    fn default_line_empty_buffer_acts_like_append() {
        let text = vec![
            "the quick brown fox".into(),
            "jumped over the lazy dog".into(),
        ];
        let mut buffer = vec![];
        let mut config = Config::default();
        let res = cmd(&text, None, &mut buffer, &mut config);
        assert!(res.is_ok());
        assert_eq!(&buffer, &text);
        assert_eq!(config.current_index, Some(1));
    }

    #[test]
    fn explicit_line_one_empty_buffer() {
        let text = vec![
            "the quick brown fox".into(),
            "jumped over the lazy dog".into(),
        ];
        let mut buffer = vec![];
        let mut config = Config::default();
        let addr = Some(LineRange::current_line());
        let res = cmd(&text, addr, &mut buffer, &mut config);
        assert!(res.is_ok());
        assert_eq!(&buffer, &text);
        assert_eq!(config.current_index, Some(1));
    }
}
