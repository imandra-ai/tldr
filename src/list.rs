use anyhow::Result;

use crate::{cli, utils};

pub fn run(cli: cli::List) -> Result<()> {
    let files = match cli.dir {
        Some(d) => {
            let mut res = vec![];
            for e in std::fs::read_dir(d)? {
                let Ok(e) = e else { continue };
                let Ok(ft) = e.file_type() else { continue };
                if ft.is_file() {
                    res.push(e.path())
                }
            }
            res
        }
        None => {
            let xdg = xdg::BaseDirectories::with_prefix(utils::XDG_PREFIX)?;
            xdg.list_data_files("")
        }
    };

    for f in files {
        let Some(f) = f.as_path().to_str() else {
            continue;
        };
        println!("{}", f)
    }

    Ok(())
}
