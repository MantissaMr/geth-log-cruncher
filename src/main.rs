
// IMPORTS
use std::path::Path;
use std::process;
use std::fs::File;
use std::io::{self, BufRead};
use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;

//DATA STRUCTURES 
#[derive(Debug)]
struct LogEntry {
    level: String,
    timestamp: String,
    message: String,
    // more fields to be added as needed 
}

//GLOBAL VARIABLES
lazy_static! {
    static ref LOG_REGEX: Regex = Regex::new(
        r"^(?P<level>INFO|WARN|ERROR|DEBUG|TRACE)\s*\[(?P<timestamp>\d{2}-\d{2}\|\d{2}:\d{2}:\d{2}(\.\d{3})?)\]\s+(?P<message>.*)"
    ).unwrap();
}

// CLI DEFINITIONS
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the log file to be processed
    log_file_path: String,
}


// PARSING LOGIC
fn parse_line(line: &str) -> Result<Option<LogEntry>, regex::Error> {
    // Core parsing logic 
    if let Some(caps) = LOG_REGEX.captures(line) {
        let entry = LogEntry{
            level: caps["level"].to_string(),
            timestamp: caps["timestamp"].to_string(),
            message: caps["message"].to_string(),
        };
        Ok(Some(entry))
    } else {
        Ok(None) 
    }
}

// ENTRY POINT
fn main() {
    // Get the user's instructions
    let cli_args = Cli::parse();

    // Pass those instructions to the main `run` function and handle any top-level errors
    if let Err(e) = run(cli_args){
        eprintln!("Application error: {}", e);
        process::exit(1)
    }
}

// WORKFLOW LOGIC
fn run(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    
    println!("Processing log file at path: {}", args.log_file_path);

    // Validation processes 
    let path = Path::new(&args.log_file_path);

    if !path.exists() {
        return Err(format!("Error: File not found at path '{}'", args.log_file_path).into());
    }

    if !path.is_file() {
        return Err(format!("Error: The path '{}' is not a file", args.log_file_path).into());
    }

    // File reading and processing
    let file = File::open(path)?;
    println!("Successfully opened the log file!");

    let reader = io::BufReader::new(file);
    
    let mut valid_line_count = 0;

    // Main processing loop
    for line_result in reader.lines() {
        let line = line_result?;

        if let Ok(Some(log_entry)) = parse_line(&line){
            valid_line_count += 1;

            if valid_line_count <= 10 {
                println!("{:?}", log_entry);
            }
        }
    }
    println!("Successful! Total valid log entries: {}", valid_line_count); 

    Ok(())
}