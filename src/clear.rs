use std::fs;

use anyhow::Result;

use crate::{cli, utils};

pub fn run(cli: cli::Clear) -> Result<()> {
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

    let mut n_deleted = 0;
    let mut n_errors = 0;

    for f in files {
        let Some(f) = f.as_path().to_str() else {
            continue;
        };

        log::debug!("removing file {f:?}");

        if let Err(err) = fs::remove_file(f) {
            log::error!("Could not remove file {f:?}: {err:?}");
            n_errors += 1;
        } else {
            n_deleted += 1;
        }
    }

    if n_errors > 0 {
        anyhow::bail!("Met {n_errors} when removing files")
    } else {
        log::info!("Removed {n_deleted} files.")
    }

    Ok(())
}
