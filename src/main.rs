use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

struct Cli {
    /// The path to the log file to be processed.
    log_file_path: String,
}

fn main() {
    let cli_args = Cli::parse();
    println!("Processing log file at path: {}", cli_args.log_file_path);
}