use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Path to a file to open
    #[arg(short = 'f', long = "file")]
    pub file_name: Option<String>,
}
