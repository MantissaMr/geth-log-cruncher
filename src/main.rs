

use std::path::Path;
use std::process;
use std::fs::File;
use std::io::{self, BufRead};


use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]

struct Cli {
    /// The path to the log file to be processed
    log_file_path: String,
}

fn main() {
    
    let cli_args = Cli::parse();

    if let Err(e) = run(cli_args){
        eprintln!("Application error: {}", e);
        process::exit(1)
    }
}

fn run(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    
    println!("Processing log file at path: {}", args.log_file_path);

    // Create a Path object from the string the user gave
    let path = Path::new(&args.log_file_path);

    // Check if the file exists at the given path 
    if !path.exists() {
        return Err(format!("Error: File not found at path '{}'", args.log_file_path).into());
    }

    // Check if the path is a file
    if !path.is_file() {
        return Err(format!("Error: The path '{}' is not a file", args.log_file_path).into());
    }

    // Open the file
    let file = File::open(path)?;

    println!("Successfully opened the log file!");

    // Create a buffered reader
    let reader = io::BufReader::new(file);
    
    let mut line_count = 0;
    for line in reader.lines() {
        let _line = line?;
        line_count += 1;
    }
    
    println!("Total number of lines in the log file: {}", line_count); 

    Ok(())
}