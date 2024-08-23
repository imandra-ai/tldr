/// Message from clients
#[derive(Clone, Debug)]
pub enum Msg<'a> {
    Empty,
    Open {
        trace_id: &'a str,
    },
    Symlink {
        /// File to symlink to with the trace file
        file: &'a str,
    },
    EmitTef {
        /// Copy the current trace as a .json, TEF formatted file in `path`
        path: &'a str,
    },
    Hardlink {
        /// File to hardlink with the trace file
        file: &'a str,
    },
    Add {
        json: &'a str,
    },
    ParseError {
        msg: &'static str,
    },
}

/// Decode a line.
pub fn decode_line<'a>(line: &'a str) -> Msg<'a> {
    use Msg::*;

    let line = line.trim();
    if line.is_empty() {
        Empty
    } else if let Some(rest) = line.strip_prefix("OPEN ") {
        Open {
            trace_id: rest.trim(),
        }
    } else if let Some(rest) = line.strip_prefix("SYMLINK ") {
        Symlink { file: rest.trim() }
    } else if let Some(rest) = line.strip_prefix("EMIT_TEF ") {
        EmitTef { path: rest.trim() }
    } else if let Some(rest) = line.strip_prefix("HARDLINK ") {
        Hardlink { file: rest.trim() }
    } else if !line.is_empty() && line.as_bytes()[0] == b'{' {
        if line.as_bytes()[line.as_bytes().len() - 1] != b'}' {
            return ParseError {
                msg: "Non closed JSON object",
            };
        }
        Add { json: line }
    } else {
        ParseError {
            msg: "Expected a valid client message",
        }
    }
}
