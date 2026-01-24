//! Z39.50 CLI client
//!
//! A command-line interface for the z3950-rs library.

use std::convert::TryInto;

use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use z3950_rs::{Client, CloseReason, Entry, TriggerRequestedAction};

#[derive(Parser)]
#[command(name = "z3950")]
#[command(about = "Z39.50 interactive client CLI", version, long_about = None)]
struct Cli {
    /// Server address (host:port)
    #[arg(short, long, default_value = "localhost:210")]
    server: String,

    /// Username for authentication
    #[arg(short, long)]
    user: Option<String>,

    /// Password for authentication
    #[arg(short, long)]
    password: Option<String>,

    /// Output format (text or json)
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,
}

#[derive(Clone, Copy, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Text,
    Json,
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
    // Connect to server
    let credentials = match (&cli.user, &cli.password) {
        (Some(u), Some(p)) => Some((u.as_str(), p.as_str())),
        _ => None,
    };

    println!("Connecting to {}...", cli.server);
    let mut client = connect(&cli.server, credentials).await?;
    println!("Connected successfully!");
    println!("Type 'help' for available commands, 'exit' or 'quit' to disconnect.");

    // Initialize REPL
    let mut rl = DefaultEditor::new().map_err(|e| {
        z3950_rs::Error::Protocol(format!("Failed to initialize readline: {e}"))
    })?;

    loop {
        let readline = rl.readline("z3950> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(line);

                // Parse and execute command
                match execute_command(&mut client, line, cli.format).await {
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
    if !client.is_closed() {
        let _ = client.close().await;
        println!("Connection closed.");
    }

    Ok(())
}

async fn execute_command(
    client: &mut Client,
    line: &str,
    format: OutputFormat,
) -> z3950_rs::Result<bool> {
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
        "exit" | "quit" | "close" => {
            if cmd == "close" && !args.is_empty() {
                let reason = args.join(" ");
                let close_reason = parse_close_reason(&reason).unwrap_or(CloseReason::Finished);
                let _ = client.close_with_reason(close_reason, None).await;
            } else {
                let _ = client.close().await;
            }
            Ok(true)
        }
        "search" => {
            if args.len() < 2 {
                eprintln!("Usage: search <database> <term> [--result-set <name>]");
                return Ok(false);
            }
            let database = args[0];
            let term = args[1..].join(" ");
            let mut result_set = None;
            if let Some(pos) = args.iter().position(|&x| x == "--result-set" || x == "-r") {
                if pos + 1 < args.len() {
                    result_set = Some(args[pos + 1].to_string());
                }
            }
            if let Some(rs) = result_set {
                client.set_result_set_name(rs.clone());
            }
            let dbs: Vec<&str> = database.split(',').map(str::trim).collect();
            let response = client.search(&dbs, &term).await?;
            let result_count: i64 = response.result_count.try_into().unwrap_or(0);

            match format {
                OutputFormat::Text => {
                    println!("Search completed");
                    println!("Result count: {}", result_count);
                    println!("Result set: {}", client.result_set_name());
                    println!("Status: {}", if response.search_status { "success" } else { "failure" });
                }
                OutputFormat::Json => {
                    let output = serde_json::json!({
                        "result_count": result_count,
                        "result_set": client.result_set_name(),
                        "search_status": response.search_status,
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                }
            }
            Ok(false)
        }
        "present" => {
            if args.len() < 2 {
                eprintln!("Usage: present <database> <term> [--start <n>] [--count <n>] [--raw]");
                return Ok(false);
            }
            let database = args[0];
            let term = args[1..].iter()
                .take_while(|&&x| !x.starts_with("--"))
                .copied()
                .collect::<Vec<_>>()
                .join(" ");
            let mut start = 1i64;
            let mut count = 10i64;
            let mut raw = false;

            let mut i = 0;
            while i < args.len() {
                match args[i] {
                    "--start" | "-s" if i + 1 < args.len() => {
                        start = args[i + 1].parse().unwrap_or(1);
                        i += 2;
                    }
                    "--count" | "-c" if i + 1 < args.len() => {
                        count = args[i + 1].parse().unwrap_or(10);
                        i += 2;
                    }
                    "--raw" => {
                        raw = true;
                        i += 1;
                    }
                    _ => i += 1,
                }
            }

            let dbs: Vec<&str> = database.split(',').map(str::trim).collect();
            let search_resp = client.search(&dbs, &term).await?;
            let result_count: i64 = search_resp.result_count.try_into().unwrap_or(0);

            if result_count == 0 {
                println!("No results found");
                return Ok(false);
            }

            if raw {
                let records = client.present_raw(start, count).await?;
                match format {
                    OutputFormat::Text => {
                        for (i, record) in records.iter().enumerate() {
                            println!("--- Record {} ({} bytes) ---", start + i as i64, record.len());
                            for chunk in record.chunks(16) {
                                let hex: Vec<String> = chunk.iter().map(|b| format!("{b:02x}")).collect();
                                let ascii: String = chunk
                                    .iter()
                                    .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
                                    .collect();
                                println!("{:<48}  {}", hex.join(" "), ascii);
                            }
                        }
                    }
                    OutputFormat::Json => {
                        let output: Vec<String> = records.iter().map(|r| base64_encode(r)).collect();
                        println!("{}", serde_json::to_string_pretty(&output).unwrap());
                    }
                }
            } else {
                let records = client.present_marc(start, count).await?;
                match format {
                    OutputFormat::Text => {
                        for (i, record) in records.iter().enumerate() {
                            println!("=== Record {} ===", start + i as i64);
                            println!("Leader: {:?}", record.leader);
                            for cf in &record.control_fields {
                                println!("  {}: {}", cf.tag, cf.value);
                            }
                            for df in &record.data_fields {
                                let subfields: Vec<String> = df.subfields.iter()
                                    .map(|sf| format!("${}{}", sf.code, sf.value))
                                    .collect();
                                println!("  {} {}{}: {}", df.tag, df.ind1, df.ind2, subfields.join(" "));
                            }
                            println!();
                        }
                    }
                    OutputFormat::Json => {
                        let output: Vec<serde_json::Value> = records.iter().map(|r| {
                            let control_fields: Vec<serde_json::Value> = r.control_fields.iter()
                                .map(|f| serde_json::json!({ "tag": f.tag, "value": f.value }))
                                .collect();
                            let data_fields: Vec<serde_json::Value> = r.data_fields.iter()
                                .map(|f| {
                                    let subfields: Vec<serde_json::Value> = f.subfields.iter()
                                        .map(|sf| serde_json::json!({
                                            "code": sf.code.to_string(),
                                            "value": sf.value
                                        }))
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
                        }).collect();
                        println!("{}", serde_json::to_string_pretty(&output).unwrap());
                    }
                }
            }
            Ok(false)
        }
        "scan" => {
            if args.len() < 2 {
                eprintln!("Usage: scan <database> <term> [--attribute <n>] [--count <n>] [--position <n>]");
                return Ok(false);
            }
            let database = args[0];
            let term = args[1..].iter()
                .take_while(|&&x| !x.starts_with("--"))
                .copied()
                .collect::<Vec<_>>()
                .join(" ");
            let mut attribute = 4i64;
            let mut count = 20i64;
            let mut position = None;

            let mut i = 0;
            while i < args.len() {
                match args[i] {
                    "--attribute" | "-a" if i + 1 < args.len() => {
                        attribute = args[i + 1].parse().unwrap_or(4);
                        i += 2;
                    }
                    "--count" | "-c" if i + 1 < args.len() => {
                        count = args[i + 1].parse().unwrap_or(20);
                        i += 2;
                    }
                    "--position" | "-p" if i + 1 < args.len() => {
                        position = Some(args[i + 1].parse().unwrap_or(0));
                        i += 2;
                    }
                    _ => i += 1,
                }
            }

            let dbs: Vec<&str> = database.split(',').map(str::trim).collect();
            let response = client.scan(&dbs, &term, attribute, count, position).await?;

            match format {
                OutputFormat::Text => {
                    println!("Scan completed - Status: {:?}", response.scan_status);
                    if let Some(entries) = Client::extract_scan_entries(&response) {
                        if let Some(ref list) = entries.entries {
                            for entry in list {
                                if let Entry::TermInfo(info) = entry {
                                    let term_str = format!("{:?}", info.term);
                                    let count: i64 = info.global_occurrences.as_ref()
                                        .and_then(|i| i.try_into().ok())
                                        .unwrap_or(0);
                                    println!("  {} ({})", term_str, count);
                                }
                            }
                        }
                    }
                    if let Some(ref pos) = response.position_of_term {
                        let pos_val: i64 = pos.try_into().unwrap_or(0);
                        println!("Position of term: {pos_val}");
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
                    let pos_val: Option<i64> = response.position_of_term.as_ref()
                        .and_then(|p| p.try_into().ok());
                    let output = serde_json::json!({
                        "scan_status": format!("{:?}", response.scan_status),
                        "entries": entries,
                        "position_of_term": pos_val,
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                }
            }
            Ok(false)
        }
        "sort" => {
            if args.len() < 3 {
                eprintln!("Usage: sort <input_result_sets> <output_result_set> <field> [--descending] [--case-insensitive]");
                return Ok(false);
            }
            let input = args[0];
            let output = args[1];
            let field = args[2];
            let descending = args.contains(&"--descending");
            let case_insensitive = args.contains(&"--case-insensitive");

            let input_sets: Vec<&str> = input.split(',').map(str::trim).collect();
            let sort_key = Client::sort_key_by_field(field, !descending, !case_insensitive);
            let response = client.sort(&input_sets, output, vec![sort_key]).await?;

            match format {
                OutputFormat::Text => {
                    println!("Sort completed - Status: {:?}", response.sort_status);
                    if let Some(ref count) = response.result_count {
                        let count_val: i64 = count.try_into().unwrap_or(0);
                        println!("Result count: {count_val}");
                    }
                }
                OutputFormat::Json => {
                    let count_val: Option<i64> = response.result_count.as_ref()
                        .and_then(|c| c.try_into().ok());
                    let output = serde_json::json!({
                        "sort_status": format!("{:?}", response.sort_status),
                        "result_count": count_val,
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                }
            }
            Ok(false)
        }
        "delete" => {
            let all = args.contains(&"--all") || args.contains(&"-a");
            let result_sets = if all {
                None
            } else if !args.is_empty() {
                Some(args.join(","))
            } else {
                eprintln!("Usage: delete [<result_sets>] [--all]");
                return Ok(false);
            };

            let response = if all || result_sets.is_none() {
                client.delete_all_result_sets().await?
            } else {
                let sets: Vec<&str> = result_sets.as_ref().unwrap().split(',').map(str::trim).collect();
                client.delete_result_sets(&sets).await?
            };

            let success = Client::delete_was_successful(&response);

            match format {
                OutputFormat::Text => {
                    println!(
                        "Delete {} - Status: {:?}",
                        if success { "succeeded" } else { "failed" },
                        response.delete_operation_status
                    );
                }
                OutputFormat::Json => {
                    let output = serde_json::json!({
                        "success": success,
                        "delete_operation_status": format!("{:?}", response.delete_operation_status),
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                }
            }
            Ok(false)
        }
        "duplicate-detection" | "dedup" => {
            if args.len() < 2 {
                eprintln!("Usage: duplicate-detection <input_result_sets> <output_result_set> [--clustering]");
                return Ok(false);
            }
            let input = args[0];
            let output = args[1];
            let clustering = args.contains(&"--clustering");

            let input_sets: Vec<&str> = input.split(',').map(str::trim).collect();
            let response = client.duplicate_detection(&input_sets, output, clustering).await?;
            let success = Client::duplicate_detection_was_successful(&response);

            match format {
                OutputFormat::Text => {
                    println!(
                        "Duplicate detection {} - Status: {:?}",
                        if success { "succeeded" } else { "failed" },
                        response.status
                    );
                    if let Some(ref count) = response.result_set_count {
                        let count_val: i64 = count.try_into().unwrap_or(0);
                        println!("Result set count: {count_val}");
                    }
                }
                OutputFormat::Json => {
                    let count_val: Option<i64> = response.result_set_count.as_ref()
                        .and_then(|c| c.try_into().ok());
                    let output = serde_json::json!({
                        "success": success,
                        "status": format!("{:?}", response.status),
                        "result_set_count": count_val,
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                }
            }
            Ok(false)
        }
        "resource-report" | "res-report" => {
            let response = client.resource_report(None, None).await?;
            let success = Client::resource_report_was_successful(&response);

            match format {
                OutputFormat::Text => {
                    println!(
                        "Resource report {} - Status: {:?}",
                        if success { "succeeded" } else { "failed" },
                        response.resource_report_status
                    );
                    if let Some(report) = Client::extract_resource_report(&response) {
                        println!("Report: {report:?}");
                    }
                }
                OutputFormat::Json => {
                    let output = serde_json::json!({
                        "success": success,
                        "status": format!("{:?}", response.resource_report_status),
                    });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                }
            }
            Ok(false)
        }
        "trigger-resource-control" | "trigger" => {
            if args.is_empty() {
                eprintln!("Usage: trigger-resource-control <action>");
                eprintln!("Actions: resourceReport, resourceControl, cancel");
                return Ok(false);
            }
            let action = parse_trigger_action(args[0])?;
            client.trigger_resource_control(action, None).await?;

            match format {
                OutputFormat::Text => println!("Trigger resource control sent"),
                OutputFormat::Json => {
                    let output = serde_json::json!({ "success": true });
                    println!("{}", serde_json::to_string_pretty(&output).unwrap());
                }
            }
            Ok(false)
        }
        "result-set" | "rs" => {
            if args.is_empty() {
                println!("Current result set: {}", client.result_set_name());
            } else {
                client.set_result_set_name(args[0]);
                println!("Result set set to: {}", client.result_set_name());
            }
            Ok(false)
        }
        _ => {
            eprintln!("Unknown command: {}. Type 'help' for available commands.", cmd);
            Ok(false)
        }
    }
}

fn print_help() {
    println!("Available commands:");
    println!("  help, ?                          - Show this help");
    println!("  exit, quit, close                - Close connection and exit");
    println!("  search <db> <term> [--result-set <name>]");
    println!("                                   - Search for records");
    println!("  present <db> <term> [--start <n>] [--count <n>] [--raw]");
    println!("                                   - Retrieve records (searches first)");
    println!("  scan <db> <term> [--attribute <n>] [--count <n>] [--position <n>]");
    println!("                                   - Browse an index");
    println!("  sort <input> <output> <field> [--descending] [--case-insensitive]");
    println!("                                   - Sort result sets");
    println!("  delete [<result_sets>] [--all]   - Delete result sets");
    println!("  duplicate-detection <input> <output> [--clustering]");
    println!("                                   - Detect duplicates");
    println!("  resource-report                  - Request resource report");
    println!("  trigger-resource-control <action> - Trigger resource control");
    println!("  result-set [<name>]              - Get/set current result set name");
}

async fn connect(server: &str, credentials: Option<(&str, &str)>) -> z3950_rs::Result<Client> {
    match credentials {
        Some(creds) => Client::connect_with_credentials(server, Some(creds)).await,
        None => Client::connect(server).await,
    }
}

fn parse_close_reason(s: &str) -> z3950_rs::Result<CloseReason> {
    Ok(match s.to_lowercase().as_str() {
        "finished" => CloseReason::Finished,
        "shutdown" => CloseReason::Shutdown,
        "systemproblem" | "system_problem" => CloseReason::SystemProblem,
        "costlimit" | "cost_limit" => CloseReason::CostLimit,
        "resources" => CloseReason::Resources,
        "securityviolation" | "security_violation" => CloseReason::SecurityViolation,
        "protocolerror" | "protocol_error" => CloseReason::ProtocolError,
        "lackofactivity" | "lack_of_activity" => CloseReason::LackOfActivity,
        "peerabort" | "peer_abort" => CloseReason::PeerAbort,
        "unspecified" => CloseReason::Unspecified,
        _ => return Err(z3950_rs::Error::Protocol(format!("Unknown close reason: {s}"))),
    })
}

fn parse_trigger_action(s: &str) -> z3950_rs::Result<TriggerRequestedAction> {
    Ok(match s.to_lowercase().as_str() {
        "resourcereport" | "resource_report" => TriggerRequestedAction::ResourceReport,
        "resourcecontrol" | "resource_control" => TriggerRequestedAction::ResourceControl,
        "cancel" => TriggerRequestedAction::Cancel,
        _ => return Err(z3950_rs::Error::Protocol(format!("Unknown trigger action: {s}"))),
    })
}

fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }
    result
}
