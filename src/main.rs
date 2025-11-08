// --- IMPORTS ---
// Standard library imports
use std::path::Path;
use std::process;
use std::fs::File;
use std::io::{self, BufRead};

// Third-party libraries
use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;
use chrono::{DateTime, Datelike, Local, NaiveDateTime};
use std::collections::HashMap;
use serde::{Serialize};
use indicatif::{ProgressBar, ProgressStyle};

// --- DATA STRUCTURES ---
/// Represents a structured log entry parsed from the input file.
#[derive(Debug, Serialize)]
struct LogEntry {
    level: String,                     // Log level (e.g., INFO, WARN, ERROR)
    timestamp: DateTime<Local>,        // Log timestamp in local timezone
    message: String,                   // Main log message
    details: HashMap<String, String>,  // Key-value pairs extracted from the message
}

/// Command-line arguments for the application.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    log_file_path: String,  // Path to the log file to process
    #[arg(long)]
    year: Option<i32>,      // Optional year for timestamps (default: current year)
}

// --- GLOBAL VARIABLES ---
// Precompiled regex patterns for efficient log parsing.

lazy_static! {
    // Regex to capture the main components of a log line
    static ref LOG_REGEX: Regex = Regex::new(
        r"^(?P<level>INFO|WARN|ERROR|DEBUG|TRACE)\s*\[(?P<timestamp>.+?)\]\s+(?P<message>.*)"
    ).unwrap();

    // Regex to capture key-value pairs in the log message
    static ref KV_REGEX: Regex = Regex::new(r#"(?P<key>\w+)=(?P<value>"[^"]*"|\S+)"#).unwrap();
}

// --- ENTRY POINT ---
/// The main entry point for the application.
fn main() {
    let cli_args = Cli::parse();

    if let Err(e) = run(cli_args) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}

// --- CORE & HELPER FUNCTIONS ---
/// The main workflow logic orchestrator for the application.
/// 
/// - Validates the input file.
/// - Sets up the progress bar.
/// - Processes the log file line by line.
/// - Outputs a run summary.
fn run(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(&args.log_file_path);
    validate_path(path)?;

    let year = args.year.unwrap_or_else(|| Local::now().year());

    let file_metadata = File::open(path)?.metadata()?;
    let total_bytes = file_metadata.len();

    // Pass total_bytes to setup_progress_bar
    let pb = setup_progress_bar(total_bytes); 
    pb.set_message("Initializing...");

    // Empty file check
    if total_bytes == 0 {
        pb.finish_with_message("File is empty.");
        eprintln!("Input file is empty. Nothing to process.");
        return Ok(());
    }

    // Process the log file and get line counts
    let (total_lines, valid_line_count) = process_log_file(path, year, &pb)?;
    
    let invalid_line_count = total_lines - valid_line_count;
    let invalid_percentage = (invalid_line_count as f64 / total_lines as f64) * 100.0;

    // Print summary
    eprintln!("\nRun Summary");
    eprintln!("---------------------");
    eprintln!("Total Lines Processed: {}", total_lines);
    eprintln!("Valid Log Entries Found: {}", valid_line_count);
    eprintln!(
        "Invalid Log Entries: {} ({:.2}% of total lines)",
        invalid_line_count, invalid_percentage
    );
    eprintln!("Year Used for Timestamps: {}", year);
    eprintln!("---------------------");

    Ok(())
}

/// The core file processing engine. Reads a file line-by-line, parses, and prints JSON.
fn process_log_file(path: &Path, year: i32, pb: &ProgressBar) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let file = File::open(path)?; 
    
    let mut reader = io::BufReader::new(file);
    
    let mut valid_line_count = 0;
    let mut total_lines = 0;
    let mut bytes_read_so_far = 0;

    let mut line_buffer = String::new(); 
    loop {
        line_buffer.clear(); 
        let bytes_read_this_line = reader.read_line(&mut line_buffer)?;
        if bytes_read_this_line == 0 {
            break; 
        }

        total_lines += 1;
        bytes_read_so_far += bytes_read_this_line; 

        // Update the progress bar with bytes read.
        pb.set_position(bytes_read_so_far as u64);
        pb.set_message(format!("Processing line {}", total_lines));


        // Parse the line and output JSON if valid
        if let Some(log_entry) = parse_line(&line_buffer, year) {
            valid_line_count += 1;
            let json_string = serde_json::to_string(&log_entry)?;
            println!("{}", json_string);
        }
    }
    
    pb.finish_with_message("Processing complete!");
    Ok((total_lines, valid_line_count))
}

/// Parses a single log line into a `LogEntry` struct.
fn parse_line(line: &str, year: i32) -> Option<LogEntry> {
    if let Some(caps) = LOG_REGEX.captures(line) {
        let raw_timestamp_str = &caps["timestamp"];
        let with_year = format!("{}-{}", year, raw_timestamp_str);
        let naive_dt = NaiveDateTime::parse_from_str(&with_year, "%Y-%m-%d|%H:%M:%S%.f").ok()?;
        let local_dt = naive_dt.and_local_timezone(Local).single()?;

        let message = caps["message"].to_string();
        let mut details = HashMap::new();
        for kv_caps in KV_REGEX.captures_iter(&message) {
            let key = kv_caps["key"].to_string();
            let mut value = kv_caps["value"].to_string();
            if value.starts_with('"') && value.ends_with('"') {
                value = value.trim_matches('"').to_string();
            }
            details.insert(key, value);
        }
            
        Some(LogEntry {
            level: caps["level"].to_string(),
            timestamp: local_dt,
            message: caps["message"].to_string(),
            details, 
        })
    } else {
        None
    }
}

/// Validates that the provided path exists and is a file.
fn validate_path(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Err(format!("Error: File not found at path '{}'", path.display()).into());
    }

    if !path.is_file() {
        return Err(format!("Error: The path '{}' is a directory, not a file", path.display()).into());
    }

    Ok(())
}

/// Sets up a bar-style progress bar for file processing, based on bytes.
fn setup_progress_bar(total_bytes: u64) -> ProgressBar {
    let pb = ProgressBar::new(total_bytes); // Progress bar based on bytes
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green}[{bar:40.cyan/blue}] {bytes}/{total_bytes} ({percent}%) {msg}")
            .expect("Invalid progress bar template")
            .progress_chars("#>-"),
    );
    pb
}