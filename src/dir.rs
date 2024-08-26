use anyhow::Result;

use crate::{cli, utils};

pub fn run(cli: cli::Dir) -> Result<()> {
    let dir = match cli.dir {
        Some(d) => d,
        None => {
            let xdg = xdg::BaseDirectories::with_prefix(utils::XDG_PREFIX)?;
            xdg.get_data_home()
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Path is not printable"))?
                .to_owned()
        }
    };

    println!("{}", dir);

    Ok(())
}
