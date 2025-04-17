use clap::Parser;

#[derive(Parser, Debug)]
pub struct CommandLine {
    #[arg(short, long, default_value = "capture.json", help = "File path of the capture.json")]
    pub filename: String,
}
