use clap::Parser;

#[derive(Parser, Debug)]
pub struct CommandLine {
    #[arg(short, long, default_value = "capture.log", help = "File path of the capture.log")]
    filename: String,
}
