use std::io::{BufRead, Write};

use anyhow::Result;

pub const XDG_PREFIX: &'static str = "tldrs";

pub fn emit_tef(reader: &mut impl BufRead, writer: &mut impl Write) -> Result<()> {
    let mut first = true;

    let mut json = String::new();
    loop {
        json.clear();
        let n = reader.read_line(&mut json)?;
        if n == 0 {
            break;
        }

        if first {
            write!(writer, "[")?;
            first = false;
        } else {
            write!(writer, ",\n")?;
        }

        write!(writer, "{}", json.trim())?;
    }
    write!(writer, "]\n")?;

    Ok(())
}
