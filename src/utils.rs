use std::io::{BufRead, Write};

use anyhow::Result;

pub const XDG_PREFIX: &'static str = "tldrs";

/// Reads jsonl from `reader` and writes a single
/// TEF-format json object into `writer`.
pub fn emit_tef(reader: &mut impl BufRead, writer: &mut impl Write) -> Result<()> {
    let mut bad_json = 0;
    let mut first = true;

    let mut json = String::new();
    loop {
        json.clear();
        let n = reader.read_line(&mut json)?;
        if n == 0 {
            break;
        }

        let json_trimmed = json.trim();
        if json_trimmed.is_empty() {
            continue;
        } else if json_trimmed.as_bytes()[0] != b'{'
            || json_trimmed.as_bytes()[json_trimmed.as_bytes().len() - 1] != b'}'
        {
            // make sure the object is not trivially invalid
            bad_json += 1;
            continue;
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

    if bad_json > 0 {
        log::warn!("Read {bad_json} invalid JSON objects while producing TEF object.");
    }

    Ok(())
}
