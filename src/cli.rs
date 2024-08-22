#[derive(Debug, clap::Parser)]
pub struct List {
    /// Storage directory
    pub dir: Option<String>,
}

#[derive(Debug, clap::Parser)]
pub struct Serve {
    /// Path to the unix socket to serve
    pub unix_socket: Option<String>,
    /// Storage directory
    pub dir: Option<String>,
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
