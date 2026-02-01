//! Z39.50 CLI client (yaz-client compatible)
//!
//! A command-line interface for the z3950-rs library.

use std::convert::TryInto;

use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use z3950_rs::{Client, Entry, QueryLanguage};

#[derive(Parser)]
#[command(name = "z3950")]
#[command(about = "Z39.50 interactive client CLI (yaz-client compatible)", version, long_about = None)]
struct Cli {
    /// Server address (host:port) - optional, can use 'open' command
    #[arg(short, long)]
    server: Option<String>,

    /// Username for authentication
    #[arg(short, long)]
    user: Option<String>,

    /// Password for authentication
    #[arg(short, long)]
    password: Option<String>,

    /// Output format (text or json)
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,

    /// Database name(s), comma-separated
    #[arg(short, long, default_value = "Default")]
    database: String,
}

#[derive(Clone, Copy, Default, Debug, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Text,
    Json,
}

struct SessionState {
    client: Option<Client>,
    database: String,
    result_set: String,
    format: OutputFormat,
    user: Option<String>,
    password: Option<String>,
}

impl SessionState {
    fn new(format: OutputFormat, database: String, user: Option<String>, password: Option<String>) -> Self {
        Self {
            client: None,
            database,
            result_set: "default".to_string(),
            format,
            user,
            password,
        }
    }

    fn is_connected(&self) -> bool {
        self.client.is_some() && !self.client.as_ref().unwrap().is_closed()
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run_interactive(cli).await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run_interactive(cli: Cli) -> z3950_rs::Result<()> {
    let mut state = SessionState::new(cli.format, cli.database, cli.user, cli.password);

    // Auto-connect if server provided
    if let Some(server) = &cli.server {
        println!("Connecting to {}...", server);
        match connect_to_server(&mut state, server).await {
            Ok(_) => println!("Connected successfully!"),
            Err(e) => {
                eprintln!("Failed to connect: {e}");
                return Err(e);
            }
        }
    } else {
        println!("Type 'open <host:port>' to connect, 'help' for commands.");
    }

    // Initialize REPL
    let mut rl = DefaultEditor::new().map_err(|e| z3950_rs::Error::Protocol(format!("Failed to initialize readline: {e}")))?;

    loop {
        let prompt = if state.is_connected() { "z3950> " } else { "z3950 (not connected)> " };
        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(line);

                // Parse and execute command
                match execute_command(&mut state, line).await {
                    Ok(should_exit) => {
                        if should_exit {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {err:?}");
                break;
            }
        }
    }

    // Close connection
    if let Some(mut client) = state.client.take() {
        if !client.is_closed() {
            let _ = client.close().await;
            println!("Connection closed.");
        }
    }

    Ok(())
}

async fn execute_command(state: &mut SessionState, line: &str) -> z3950_rs::Result<bool> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(false);
    }

    let cmd = parts[0].to_lowercase();
    let args = &parts[1..];

    match cmd.as_str() {
        "help" | "?" => {
            print_help();
            Ok(false)
        }
        "quit" | "exit" => {
            if let Some(mut client) = state.client.take() {
                if !client.is_closed() {
                    let _ = client.close().await;
                }
            }
            Ok(true)
        }
        "open" => {
            if args.is_empty() {
                eprintln!("Usage: open <host:port>");
                return Ok(false);
            }
            let server = args[0];
            connect_to_server(state, server).await?;
            println!("Connected to {}", server);
            Ok(false)
        }
        "close" => {
            if let Some(mut client) = state.client.take() {
                if !client.is_closed() {
                    let _ = client.close().await;
                    println!("Connection closed.");
                }
            } else {
                println!("Not connected.");
            }
            Ok(false)
        }
        "find" => {
            if !state.is_connected() {
                eprintln!("Not connected. Use 'open <host:port>' first.");
                return Ok(false);
            }
            if args.is_empty() {
                eprintln!("Usage: find <query>");
                return Ok(false);
            }
            let query = args.join(" ");
            let client = state.client.as_mut().unwrap();
            let dbs: Vec<&str> = state.database.split(',').map(str::trim).collect();
            println!("query: {:?}", query);
            println!("cql: {:?}", QueryLanguage::CQL(query.clone()));
            let response = client.search(&dbs, QueryLanguage::CQL(query)).await?;
            let result_count: i64 = response.result_count.try_into().unwrap_or(0);

            match state.format {
                OutputFormat::Text => {
                    println!("Records: {}", result_count);
                    println!("Result set: {}", state.result_set);
                }
                OutputFormat::Json => {
                    let output = serde_json::json!({
                        "result_count": result_count,
                        "result_set": state.result_set,
                        "search_status": response.search_status,
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                }
            }
            Ok(false)
        }
        "show" => {
            if !state.is_connected() {
                eprintln!("Not connected. Use 'open <host:port>' first.");
                return Ok(false);
            }
            let mut start = 1i64;
            let mut count = 10i64;

            // Parse optional start and count
            if !args.is_empty() {
                if let Ok(n) = args[0].parse::<i64>() {
                    start = n;
                    if args.len() > 1 {
                        if let Ok(n) = args[1].parse::<i64>() {
                            count = n;
                        }
                    }
                }
            }

            let client = state.client.as_mut().unwrap();
            let records = client.present_marc(start, count).await?;

            match state.format {
                OutputFormat::Text => {
                    for (i, record) in records.iter().enumerate() {
                        println!("=== Record {} ===", start + i as i64);
                        println!("Leader: {:?}", record.leader);
                        for cf in &record.control_fields {
                            println!("  {}: {}", cf.tag, cf.value);
                        }
                        for df in &record.data_fields {
                            let subfields: Vec<String> = df.subfields.iter().map(|sf| format!("${}{}", sf.code, sf.value)).collect();
                            println!("  {} {}{}: {}", df.tag, df.ind1, df.ind2, subfields.join(" "));
                        }
                        println!();
                    }
                }
                OutputFormat::Json => {
                    let output: Vec<serde_json::Value> = records
                        .iter()
                        .map(|r| {
                            let control_fields: Vec<serde_json::Value> = r.control_fields.iter().map(|f| serde_json::json!({ "tag": f.tag, "value": f.value })).collect();
                            let data_fields: Vec<serde_json::Value> = r
                                .data_fields
                                .iter()
                                .map(|f| {
                                    let subfields: Vec<serde_json::Value> = f
                                        .subfields
                                        .iter()
                                        .map(|sf| {
                                            serde_json::json!({
                                                "code": sf.code.to_string(),
                                                "value": sf.value
                                            })
                                        })
                                        .collect();
                                    serde_json::json!({
                                        "tag": f.tag,
                                        "ind1": f.ind1.to_string(),
                                        "ind2": f.ind2.to_string(),
                                        "subfields": subfields
                                    })
                                })
                                .collect();
                            serde_json::json!({
                                "leader": format!("{:?}", r.leader),
                                "control_fields": control_fields,
                                "data_fields": data_fields
                            })
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                }
            }
            Ok(false)
        }
        "scan" => {
            if !state.is_connected() {
                eprintln!("Not connected. Use 'open <host:port>' first.");
                return Ok(false);
            }
            if args.is_empty() {
                eprintln!("Usage: scan <term>");
                return Ok(false);
            }
            let term = args.join(" ");
            let client = state.client.as_mut().unwrap();
            let dbs: Vec<&str> = state.database.split(',').map(str::trim).collect();
            let response = client.scan(&dbs, &term, 4, 20, None).await?;

            match state.format {
                OutputFormat::Text => {
                    if let Some(entries) = Client::extract_scan_entries(&response) {
                        if let Some(ref list) = entries.entries {
                            for entry in list {
                                if let Entry::TermInfo(info) = entry {
                                    let term_str = format!("{:?}", info.term);
                                    let count: i64 = info.global_occurrences.as_ref().and_then(|i| i.try_into().ok()).unwrap_or(0);
                                    println!("  {} ({})", term_str, count);
                                }
                            }
                        }
                    }
                }
                OutputFormat::Json => {
                    let entries: Vec<serde_json::Value> = Client::extract_scan_entries(&response)
                        .and_then(|e| e.entries.as_ref())
                        .map(|list| {
                            list.iter()
                                .filter_map(|e| match e {
                                    Entry::TermInfo(info) => Some(serde_json::json!({
                                        "term": format!("{:?}", info.term),
                                        "global_occurrences": info.global_occurrences.as_ref()
                                            .and_then(|i| TryInto::<i64>::try_into(i).ok()),
                                    })),
                                    _ => None,
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let output = serde_json::json!({
                        "scan_status": format!("{:?}", response.scan_status),
                        "entries": entries,
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                }
            }
            Ok(false)
        }
        "set" => {
            if args.len() < 2 {
                eprintln!("Usage: set <option> <value>");
                eprintln!("Options: database, result-set, format");
                return Ok(false);
            }
            let option = args[0].to_lowercase();
            let value = args[1];

            match option.as_str() {
                "database" | "db" => {
                    state.database = value.to_string();
                    println!("Database set to: {}", state.database);
                }
                "result-set" | "rs" => {
                    state.result_set = value.to_string();
                    if let Some(ref mut client) = state.client {
                        client.set_result_set_name(&state.result_set);
                    }
                    println!("Result set set to: {}", state.result_set);
                }
                "format" => {
                    match value.to_lowercase().as_str() {
                        "text" => state.format = OutputFormat::Text,
                        "json" => state.format = OutputFormat::Json,
                        _ => {
                            eprintln!("Invalid format. Use 'text' or 'json'");
                            return Ok(false);
                        }
                    }
                    println!(
                        "Format set to: {}",
                        match state.format {
                            OutputFormat::Text => "text",
                            OutputFormat::Json => "json",
                        }
                    );
                }
                _ => {
                    eprintln!("Unknown option: {}. Use: database, result-set, format", option);
                    return Ok(false);
                }
            }
            Ok(false)
        }
        "get" => {
            if args.is_empty() {
                eprintln!("Usage: get <option>");
                eprintln!("Options: database, result-set, format");
                return Ok(false);
            }
            let option = args[0].to_lowercase();
            match option.as_str() {
                "database" | "db" => println!("Database: {}", state.database),
                "result-set" | "rs" => println!("Result set: {}", state.result_set),
                "format" => println!(
                    "Format: {}",
                    match state.format {
                        OutputFormat::Text => "text",
                        OutputFormat::Json => "json",
                    }
                ),
                _ => {
                    eprintln!("Unknown option: {}. Use: database, result-set, format", option);
                    return Ok(false);
                }
            }
            Ok(false)
        }
        _ => {
            eprintln!("Unknown command: {}. Type 'help' for available commands.", cmd);
            Ok(false)
        }
    }
}

async fn connect_to_server(state: &mut SessionState, server: &str) -> z3950_rs::Result<()> {
    // Close existing connection if any
    if let Some(mut client) = state.client.take() {
        if !client.is_closed() {
            let _ = client.close().await;
        }
    }

    let credentials = match (&state.user, &state.password) {
        (Some(u), Some(p)) => Some((u.as_str(), p.as_str())),
        _ => None,
    };

    let client = match credentials {
        Some(creds) => Client::connect_with_credentials(server, Some(creds)).await?,
        None => Client::connect(server).await?,
    };

    state.client = Some(client);
    if let Some(ref mut client) = state.client {
        client.set_result_set_name(&state.result_set);
    }

    Ok(())
}

fn print_help() {
    println!("Available commands:");
    println!("  open <host:port>                  - Connect to Z39.50 server");
    println!("  close                             - Close connection");
    println!("  find <query>                      - Search for records");
    println!("  show [start] [count]              - Display records (default: 1 10)");
    println!("  scan <term>                       - Browse index");
    println!("  set <option> <value>              - Set option (database, result-set, format)");
    println!("  get <option>                      - Get option value");
    println!("  help, ?                           - Show this help");
    println!("  quit, exit                        - Exit");
}
