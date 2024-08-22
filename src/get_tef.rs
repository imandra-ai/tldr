use std::{
    fs,
    io::{self, stdout, BufReader, BufWriter},
};

use anyhow::Result;

use crate::{cli, utils};

pub fn run(cli: cli::GetTEF) -> Result<()> {
    let file_in = fs::File::open(cli.jsonl_file)?;
    let mut reader = BufReader::new(file_in);

    let out: Box<dyn io::Write> = match cli.o {
        Some(f) => Box::new(fs::File::create(f)?),
        None => Box::new(stdout().lock()),
    };
    let mut writer = BufWriter::new(out);

    utils::emit_tef(&mut reader, &mut writer)?;

    Ok(())
}
