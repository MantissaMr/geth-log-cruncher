# Geth Log Cruncher

A blazingly simple but powerful command-line tool for parsing unstructured Ethereum Geth logs into clean, machine-readable JSONL, to transform messy log files into structured data, perfect for analysis, db ingestion, or processing with CLI tools like `jq`.

## Installation

tk

## Usage

### Basic example

```bash
geth-log-cruncher /path/to/your/geth.log > parsed_logs.jsonl
```

### Specifying a year

For archived log files where the timestamp year isn't present or isn't the current year, use `--year`:

```bash
geth-log-cruncher /path/to/archive/geth-2023.log --year 2023 > parsed_2023_logs.jsonl
```

### Filtering with jq

Because the tool outputs JSONL to `stdout`, you can filter on the fly. Example: show only `ERROR`-level logs:

```bash
geth-log-cruncher /path/to/your/geth.log | jq 'select(.level == "ERROR")'
```

### Example pipeline

Write parsed output to a file, then count ERRORs:

```bash
geth-log-cruncher /var/log/geth.log > /tmp/geth.jsonl
jq -r '.level' /tmp/geth.jsonl | grep -c ERROR
```

## Output format
Typical fields:

* `timestamp` — ISO 8601 timestamp (reconstructed using `--year` when needed).  
* `level` — log level when present (e.g., `INFO`, `WARN`, `ERROR`).  
* `target` — the module or subsystem that emitted the log (when available).  
* `message` — the raw log message text.  
* `details` — an object of parsed KV pairs extracted from the message (flexible and sparse).

## Examples
A sample parsed line might look like:

```json
{
  "timestamp": "2023-07-01T12:34:56Z",
  "level": "ERROR",
  "target": "eth/downloader",
  "message": "failed to download block",
  "details": {
    "block": "0xabc123",
    "peer": "12D3K..."
  }
}
```

## Philosophy
Keep your data stream clean. The tool writes only machine-readable JSONL to `stdout`, and all human-facing progress or summary messages are emitted to `stderr`. This makes it composable with standard UNIX tools and ideal for automated ingestion pipelines.

## Contributing
PRs and issues welcome. If you add parsers for more Geth subsystems or improve performance, please open an issue first to discuss the approach. Even better doc welcome of course.

## License
tk
```