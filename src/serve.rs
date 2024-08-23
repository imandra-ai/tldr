use std::{
    collections::{
        hash_map::{self},
        HashMap,
    },
    fs,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    os::unix::net::UnixListener,
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{self, AtomicBool},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crate::{cli, msg, utils};
use anyhow::{Context, Result};
use daemonize::Daemonize;

/// A trace ID, used to coordinate logs/traces from multiple processes.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct TraceID(String);

impl<'a> From<&'a str> for TraceID {
    fn from(value: &'a str) -> Self {
        TraceID(value.to_string())
    }
}

struct TraceFile {
    trace_id: TraceID,
    /// The pathbuf of the trace file
    path: PathBuf,
    /// Opened file descriptor to the file
    out: Mutex<BufWriter<fs::File>>,
}

struct State {
    active: AtomicBool,
    die_when_idle: AtomicBool,
    /// User might have provided a single file into which all traces go
    into_file: Option<String>,
    socket_path: PathBuf,
    dir: PathBuf,
    files: Mutex<HashMap<TraceID, Arc<TraceFile>>>,
}

impl Drop for State {
    fn drop(&mut self) {
        // remove socket file
        log::debug!("removing socket file {:?}", self.socket_path);
        let _ = fs::remove_file(&self.socket_path);

        self.close_all_force();
    }
}

impl State {
    /// path for this trace
    fn trace_file_path(&self, trace_id: &TraceID) -> PathBuf {
        let mut path = self.dir.clone();
        {
            let filename = format!("{}.jsonl", trace_id.0);
            path.push(filename);
        }
        path
    }

    fn get_trace_file(&self, trace_id: impl Into<TraceID>) -> Result<Arc<TraceFile>> {
        let trace_id = trace_id.into();

        let mut files = self.files.lock().unwrap();
        let trf = match files.entry(trace_id.clone()) {
            hash_map::Entry::Occupied(trf) => trf.get().clone(),
            hash_map::Entry::Vacant(e) => {
                let path = match self.into_file.as_ref() {
                    None => self.trace_file_path(&trace_id),
                    Some(p) => PathBuf::from_str(&p)?,
                };

                let file = fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .write(true)
                    .open(&path)?;
                let out = BufWriter::new(file);
                let trf = Arc::new(TraceFile {
                    trace_id,
                    path,
                    out: Mutex::new(out),
                });

                e.insert(trf.clone());
                trf
            }
        };
        Ok(trf)
    }

    fn close_all_force(&self) {
        let mut files = self.files.lock().unwrap();
        for (_, f) in files.drain() {
            if let Err(err) = {
                let mut out = f.out.lock().unwrap();
                out.flush()
            } {
                log::error!("Error while flushing {:?}: {:?}", f.path, err)
            }
        }
    }

    fn kill(&self) {
        // try to exit gracefully
        self.active.store(false, atomic::Ordering::SeqCst);
        self.close_all_force();

        // if we don't exit in 10s, die less cleanly
        thread::spawn(|| {
            thread::sleep(Duration::from_secs(10));
            log::warn!("timeout, dying the hard way");
            std::process::exit(1);
        });
    }
}

impl TraceFile {
    fn emit_tef(&self, path: PathBuf, len: u64) -> Result<()> {
        log::info!(
            "Emit a TEF trace into {path:?} for {len} bytes of trace {:?}",
            &self.trace_id
        );

        // open trace file, read at most `len` bytes from it
        let file_in = fs::File::open(&self.path)?.take(len);
        let mut reader = BufReader::new(file_in);

        // open output TEF file
        let file_out = fs::File::create(&path)?;
        let mut writer = BufWriter::with_capacity(16 * 1024, file_out);

        utils::emit_tef(&mut reader, &mut writer)
    }
}

fn handle_client(st: Arc<State>, mut client: impl BufRead) -> Result<()> {
    let mut trace_file: Option<Arc<TraceFile>> = None;
    if st.into_file.is_some() {
        // default file, we assume the "default" trace
        let trace_id = TraceID("default".to_string());
        trace_file = Some(st.get_trace_file(trace_id)?);
    }

    let mut n_errors = 0;

    let mut line = String::new();
    loop {
        if !st.active.load(atomic::Ordering::SeqCst) {
            break;
        }

        line.clear();
        let msg = match client.read_line(&mut line) {
            Err(e) => {
                log::debug!("read_line failed: {e:?}");
                break;
            }
            Ok(0) => break, // EOF
            Ok(_) => msg::decode_line(&line),
        };

        log::debug!("got msg {:?}", &msg);
        match msg {
            msg::Msg::Empty => (),
            msg::Msg::Die => {
                log::info!("client asked us to quit");
                st.kill();
                break;
            }
            msg::Msg::DieWhenIdle => {
                st.die_when_idle.store(true, atomic::Ordering::SeqCst);
            }
            msg::Msg::Open { trace_id } => {
                log::debug!("Opening trace file for trace_id={trace_id:?}");
                trace_file = Some(st.get_trace_file(trace_id)?);
            }
            msg::Msg::Symlink { file: _ } => todo!(),
            msg::Msg::Hardlink { file: _ } => todo!(),
            msg::Msg::Add { json } => {
                let trf = trace_file
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("No trace file defined"))?;

                let mut out = trf.out.lock().unwrap();
                writeln!(out, "{}", json)?;
            }
            msg::Msg::EmitTef { path } => {
                let path: PathBuf = PathBuf::from_str(path)?;
                let trf = trace_file
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("No trace file defined"))?
                    .clone();

                // flush file, measure how long it is
                let len: u64 = {
                    let mut out = trf.out.lock().unwrap();
                    out.flush()?;

                    let file = out.get_ref();
                    file.metadata().unwrap().len()
                };

                // emit file in the background
                thread::spawn(move || {
                    if let Err(e) = trf.emit_tef(path, len) {
                        log::error!(
                            "Error when emitting a TEF file for trace {:?}: {e:?}",
                            &trf.trace_id
                        )
                    }
                });
            }
            msg::Msg::ParseError { msg } => {
                log::error!("Invalid message: {} in line {:?}", msg, line);
                n_errors += 1;
            }
        }
    }

    if n_errors > 0 {
        log::warn!("Client exiting (met {n_errors} parsing errors)");
    } else {
        log::debug!("Client exiting (no parsing errors)");
    }

    if let Some(tr) = trace_file {
        // flush on exit
        let mut out = tr.out.lock().unwrap();
        out.flush().context("flushing trace file")?;
    }

    Ok(())
}

/// Regularly flush files and close unused files.
fn cleaner_thread(st: Arc<State>) {
    while st.active.load(atomic::Ordering::SeqCst) {
        // collect copies of all files
        let mut files = vec![];

        // collect alive files in `files`, cleanup the others
        {
            let mut dead_files = vec![];

            let mut tbl = st.files.lock().unwrap();
            for (_, file) in tbl.iter() {
                if Arc::strong_count(&file) == 1 {
                    // only copy of `f`, no client is currently using it
                    dead_files.push(file.clone());
                } else {
                    files.push(file.clone())
                }
            }

            // remove and flush all these files now, before a client has  the
            // opportunity to start writing to them
            for file in dead_files {
                tbl.remove(&file.trace_id);
                log::info!("Closing file for trace_id={:?}", file.trace_id);

                if let Err(err) = {
                    let mut out = file.out.lock().unwrap();
                    out.flush()
                } {
                    log::error!("Error while flushing {:?}: {:?}", file.path, err)
                }
            }
        }

        // no active files,
        if files.is_empty() && st.die_when_idle.load(atomic::Ordering::SeqCst) {
            log::info!("No client and die_when_idle=true, exiting");
            st.kill();
        }

        for f in files {
            if let Err(err) = {
                // flush f
                let mut out = f.out.lock().unwrap();
                out.flush()
            } {
                log::error!("Error while flushing file {:?}: {:?}", f.path, err)
            }
        }

        thread::sleep(Duration::from_secs(2));
    }
}

pub fn run(cli: cli::Serve) -> Result<()> {
    if cli.daemonize {
        Daemonize::new().start().context("daemonizing")?;
    }

    let dir: PathBuf = match cli.dir {
        None => {
            let xdg = xdg::BaseDirectories::with_prefix(utils::XDG_PREFIX)?;
            let d = xdg.create_data_directory("")?;
            d
        }
        Some(d) => {
            let d = PathBuf::from_str(&d)?;
            d
        }
    };

    log::info!("data directory is {:?}", &dir);

    let socket_path: PathBuf = match cli.unix_socket {
        Some(d) => PathBuf::from_str(&d)?,
        None => {
            let mut path = std::env::temp_dir();
            path.push("tldrs.socket");
            path
        }
    };

    log::info!("serving on unix socket {socket_path:?}");

    // try to remove file, if already present
    let _ = std::fs::remove_file(&socket_path);

    // shared state
    let st = Arc::new(State {
        active: AtomicBool::new(true),
        die_when_idle: AtomicBool::new(false),
        into_file: cli.single_file.clone(),
        socket_path: socket_path.clone(),
        dir,
        files: Mutex::new(HashMap::new()),
    });

    thread::spawn({
        let st2 = st.clone();
        move || {
            cleaner_thread(st2);
        }
    });

    let listener = UnixListener::bind(&socket_path)?;
    loop {
        let (client, client_addr) = match listener.accept() {
            Ok(x) => x,
            Err(err) => {
                log::info!("could not accept more clients: {:?}", err);
                break;
            }
        };

        let st2 = st.clone();
        thread::spawn(move || {
            let client = BufReader::new(client);
            if let Err(e) = handle_client(st2, client) {
                log::error!("while handling client on {client_addr:?}, got error: {e:?}")
            }
        });
    }

    st.active.store(false, atomic::Ordering::SeqCst);
    Ok(())
}
