#[derive(Debug, clap::Parser)]
pub struct List {
    /// Storage directory
    #[arg(short = 'd', long = "dir")]
    pub dir: Option<String>,
}

#[derive(Debug, clap::Parser)]
pub struct Serve {
    /// Path to the unix socket to serve
    #[arg(long = "socket")]
    pub unix_socket: Option<String>,
    /// Storage directory
    #[arg(short = 'd', long = "dir")]
    pub dir: Option<String>,
    /// Daemonize on startup
    #[arg(long = "daemonize")]
    pub daemonize: bool,
}

#[derive(Debug, clap::Parser)]
pub struct GetTEF {
    #[arg(index = 1, value_name = "FILE")]
    pub jsonl_file: String,
    /// Output file (.json file)
    #[arg(short = 'o', long = "out")]
    pub o: Option<String>,
}

#[derive(Debug, clap::Parser)]
pub enum Command {
    /// List log files
    List(List),

    /// Serve as a daemon
    Serve(Serve),

    /// get a file as a TEF file
    GetTEF(GetTEF),
}
