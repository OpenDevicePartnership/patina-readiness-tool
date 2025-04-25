use clap::Parser;

#[derive(Default, Parser, Debug)]
pub struct CommandLine {
    #[arg(short, long, help = "File path of the capture.json")]
    pub filename: Option<String>,
}
