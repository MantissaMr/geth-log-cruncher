
// IMPORTS
use std::path::Path;
use std::process;
use std::fs::File;
use std::io::{self, BufRead};
use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;
use chrono::{DateTime, Datelike, Local, NaiveDateTime};

//DATA STRUCTURES 
#[derive(Debug)]
struct LogEntry {
    level: String,
    timestamp: DateTime<Local>,
    message: String,
    // more fields to be added as needed 
}

//GLOBAL VARIABLES
lazy_static! {
    static ref LOG_REGEX: Regex = Regex::new(
        r"^(?P<level>INFO|WARN|ERROR|DEBUG|TRACE)\s*\[(?P<timestamp>.+?)\]\s+(?P<message>.*)"
    ).unwrap();
}

// CLI DEFINITIONS
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the log file to be processed
    log_file_path: String,

    /// Optional year for log timestamps (default: current year)
    #[arg(long)]
    year: Option<i32>,
}


// PARSING LOGIC
fn parse_line(line: &str, year: i32) -> Option<LogEntry> {
    
    // Core parsing logic 
    LOG_REGEX.captures(line).and_then(|caps| {
        // Extract the raw timestamp string from the regex capture
        let raw_timestamp_str = &caps["timestamp"];

        // Prepend the year to the raw timestamp string
        let with_year = format!("{}-{}", year, raw_timestamp_str);
        
        // Use chrono to parse the timestamp with the year
        let naive_dt = NaiveDateTime::parse_from_str(&with_year, "%Y-%m-%d|%H:%M:%S%.f").ok()?;

        // Convert NaiveDateTime to DateTime<Local>
        let local_dt = naive_dt.and_local_timezone(Local).single()?;

        // Construct and return the LogEntry
        Some(LogEntry{
            level: caps["level"].to_string(),
            timestamp: local_dt,
            message: caps["message"].to_string(),

        })
    })
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
        return Err(format!("Error: The path '{}' is a directory, not a file", args.log_file_path).into());
    }

    // Determine the year to use for log entries
    let year = args.year.unwrap_or_else(|| Local::now().year());
    println!("Using year: {}", year);

    // File reading and processing
    let file = File::open(path)?;
    println!("Successfully opened the log file!");

    let reader = io::BufReader::new(file);
    
    let mut valid_line_count = 0;

    // Main processing loop
    for line_result in reader.lines() {
        let line = line_result?;

        if let Some(log_entry) = parse_line(&line, year){
            valid_line_count += 1;

            if valid_line_count <= 10 {
                println!("{:?}", log_entry);
            }
        }
    }
    println!("Successful! Total valid log entries: {}", valid_line_count); 

    Ok(())
}