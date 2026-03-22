# z3950-rs

[![crates.io](https://img.shields.io/crates/v/z3950-rs.svg)](https://crates.io/crates/z3950-rs)
[![docs.rs](https://docs.rs/z3950-rs/badge.svg)](https://docs.rs/z3950-rs)

Minimal asynchronous ([Tokio](https://tokio.rs)) Z39.50 client with MARC parsing via [`marc-rs`](https://crates.io/crates/marc-rs). PDUs are encoded/decoded in ASN.1 BER using [`rasn`](https://crates.io/crates/rasn).

## Features

- Fully async Z39.50 client built on Tokio
- Type-1 (RPN/BIB-1) search with CQL query language support
- MARC record retrieval and parsing via `marc-rs`
- Index scan/browse
- Result set sorting and duplicate detection
- Extended services, resource control, and access control
- Optional interactive CLI client (feature `cli`)

## Installation

```toml
[dependencies]
z3950-rs = "0.0.6"
```

For the CLI binary:

```bash
cargo install z3950-rs --features cli
```

## Client API

All operations are methods on `Client`. The client holds a TCP connection and a current result set name (default: `"default"`).

### Connecting

```rust
use z3950_rs::Client;

// Without credentials
let mut client = Client::connect("z3950.loc.gov:7090").await?;

// With credentials
let mut client = Client::connect_with_credentials(
    "z3950.loc.gov:7090",
    Some(("username", "password")),
).await?;
```

Performs the Z39.50 Init handshake automatically.

---

### Search

```rust
use z3950_rs::{Client, QueryLanguage};

let response = client.search(&["Voyager"], QueryLanguage::CQL("title = rust")).await?;
println!("Found {} records", response.result_count);
```

`search` accepts any type that implements `Into<Query>`. Supported inputs:

| Input type | Example |
|---|---|
| `&str` / `String` | `"rust programming"` — treated as a raw keyword term |
| `QueryLanguage::CQL(...)` | `QueryLanguage::CQL("title = rust AND dc.date >= 2020")` |
| `Query` (raw RPN) | Manual PDU construction |

The `databases` slice maps to one or more database names on the server. The result is stored server-side under the current result set name.

---

### Retrieve MARC Records

```rust
// Present and parse MARC records (positions are 1-based)
let records: Vec<z3950_rs::MarcRecord> = client.present_marc(1, 10).await?;
for record in &records {
    println!("{:?}", record);
}

// Present raw bytes (for custom parsing)
let raw: Vec<u8> = client.present_raw(1, 10).await?;
```

`present_marc` wraps `present_raw` and parses the binary MARC stream using `marc-rs`. `MarcRecord` is a re-export of `marc_rs::Record`.

---

### Scan (Index Browse)

Browse terms in a server index:

```rust
use z3950_rs::Client;

let response = client.scan(
    &["Voyager"],   // databases
    "rust",         // starting term
    4,              // BIB-1 attribute type: 4 = title
    20,             // number of terms to retrieve
    Some(1),        // preferred position of starting term (None = server default)
).await?;

if Client::scan_was_successful(&response) {
    if let Some(entries) = Client::extract_scan_entries(&response) {
        for entry in &entries.entries {
            println!("{:?}", entry);
        }
    }
}
```

Common BIB-1 Use attribute values for `attribute_type`:

| Value | Meaning |
|---|---|
| 1 | Personal name |
| 4 | Title |
| 7 | ISBN |
| 21 | Subject |
| 1003 | Author |
| 1018 | Publisher |

---

### Sort

Sort a result set by one or more fields:

```rust
let key = Client::sort_key_by_field("title", true, false);
// (field_name, ascending, case_sensitive)

let response = client.sort(
    &["default"],       // input result sets
    "sorted",           // output result set name
    vec![key],
).await?;

if Client::sort_was_successful(&response) {
    println!("Sort completed");
}
```

---

### Delete Result Sets

```rust
// Delete specific result sets by name
let response = client.delete_result_sets(&["default", "sorted"]).await?;

// Delete all result sets on the server
let response = client.delete_all_result_sets().await?;

if Client::delete_was_successful(&response) {
    println!("Deleted successfully");
}
```

---

### Duplicate Detection

```rust
let response = client.duplicate_detection(
    &["set1", "set2"],  // input result sets
    "deduped",          // output result set
    true,               // cluster duplicates together
).await?;

if Client::duplicate_detection_was_successful(&response) {
    println!("Deduplication done");
}
```

---

### Extended Services

```rust
use z3950_rs::{ExtendedServicesFunction, WaitAction};
use rasn::types::ObjectIdentifier;

let oid = ObjectIdentifier::new_unchecked(vec![1, 2, 840, 10003, 9, 1].into());

let response = client.extended_services(
    ExtendedServicesFunction::Create,
    oid,
    Some("my-package"),
    None,                        // no task-specific parameters
    WaitAction::WaitForCompletion,
).await?;

if Client::extended_services_was_successful(&response) {
    println!("Extended service executed");
}
```

---

### Resource Control

```rust
// Respond to a resource control request from the server
client.send_resource_control_response(true, Some(true)).await?;
// (continue_flag, result_set_wanted)

// Trigger a resource control action
use z3950_rs::TriggerRequestedAction;
client.trigger_resource_control(TriggerRequestedAction::Cancel, None).await?;
```

---

### Resource Report

```rust
let response = client.resource_report(None, None).await?;

if Client::resource_report_was_successful(&response) {
    if let Some(report) = Client::extract_resource_report(&response) {
        println!("{:?}", report);
    }
}
```

---

### Access Control

```rust
// Respond to an access control challenge from the server
client.send_access_control_response(b"my-token").await?;
```

---

### Close

```rust
// Graceful close (reason: Finished)
client.close().await?;

// Close with a specific reason
use z3950_rs::CloseReason;
client.close_with_reason(CloseReason::SystemProblem, Some("timeout")).await?;

println!("Closed: {}", client.is_closed());
```

---

### Result Set Management

```rust
// Read or change the active result set name
println!("{}", client.result_set_name());  // "default"
client.set_result_set_name("my-results");
```

---

## CQL Query Language

The `QueryLanguage::CQL` input accepts a subset of CQL (Contextual Query Language) and converts it to Z39.50 Type-1 RPN with BIB-1 attributes automatically.

### Syntax

```
index relation "value"
```

**Relations:** `=`, `<`, `<=`, `>`, `>=`, `<>`

**Boolean operators:** `AND`, `OR`, `NOT` (uppercase required)

**Grouping:** parentheses `(` `)`

### Supported Indexes

| Index | Alias | BIB-1 Use |
|---|---|---|
| `dc.title` | `title`, `t` | 4 |
| `dc.creator` | `author`, `a` | 1003 |
| `dc.subject` | `subject`, `s` | 21 |
| `dc.date` | `date`, `d` | 31 |
| `dc.identifier` | `isbn` | 7 |
| `dc.publisher` | `publisher` | 1018 |
| `dc.language` | `language` | 54 |
| `dc.type` | `type` | 1016 |
| `dc.description` | `description` | 62 |
| `dc.contributor` | `contributor` | 1004 |
| Numeric string | e.g. `"4"` | used directly |

### Examples

```rust
use z3950_rs::QueryLanguage;

// Simple title search
QueryLanguage::CQL("title = rust")

// Author search with quoted value
QueryLanguage::CQL(r#"dc.creator = "Knuth""#)

// Boolean AND
QueryLanguage::CQL("title = rust AND dc.date >= 2020")

// Boolean OR
QueryLanguage::CQL("title = rust OR title = cargo")

// NOT
QueryLanguage::CQL("NOT title = python")

// Grouped expression
QueryLanguage::CQL("(title = rust OR title = cargo) AND dc.date >= 2020")

// Comparison on date
QueryLanguage::CQL("dc.date > 2019")
```

---

## Full Example

```rust
use z3950_rs::{Client, QueryLanguage};
use std::convert::TryInto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect_with_credentials(
        "z3950.loc.gov:7090",
        Some(("user", "password")),
    ).await?;

    // Search
    let search = client
        .search(&["Voyager"], QueryLanguage::CQL("title = rust AND dc.date >= 2020"))
        .await?;

    let count: i64 = search.result_count.try_into()?;
    println!("Found {} records", count);

    // Retrieve MARC records
    let records = client.present_marc(1, count.min(10)).await?;
    for record in &records {
        println!("{:?}", record);
    }

    // Browse the title index
    let scan = client.scan(&["Voyager"], "rust", 4, 10, None).await?;
    if Client::scan_was_successful(&scan) {
        if let Some(entries) = Client::extract_scan_entries(&scan) {
            for entry in &entries.entries {
                println!("{:?}", entry);
            }
        }
    }

    // Sort results
    let key = Client::sort_key_by_field("title", true, false);
    client.sort(&["default"], "sorted", vec![key]).await?;

    // Clean up
    client.delete_all_result_sets().await?;
    client.close().await?;

    Ok(())
}
```

Run the bundled example:

```bash
cargo run --example search -- \
  --host z3950.loc.gov --port 7090 --db Voyager \
  --query "title = rust" --user foo --password bar
```

---

## CLI Client

An interactive CLI client compatible with `yaz-client` command conventions.

### Usage

```bash
# Interactive session
z3950 -s localhost:9999 -d Default

# With authentication
z3950 -s localhost:9999 -d Default -u username -p password

# JSON output
z3950 -s localhost:9999 -d Default --format json
```

### Available Commands

| Command | Description |
|---|---|
| `open <host:port>` | Connect to a Z39.50 server |
| `close` | Close the current connection |
| `find <query>` | Search for records |
| `show [start] [count]` | Display records (default: `show 1 10`) |
| `scan <term>` | Browse index starting from a term |
| `set database <name>` | Set database name(s), comma-separated |
| `set result-set <name>` | Set result set name |
| `set format text\|json` | Set output format |
| `get <option>` | Get current option value |
| `help` / `?` | Show help |
| `quit` / `exit` | Exit |

### Example Session

```
$ z3950 -s localhost:9999 -d Default
Connecting to localhost:9999...
Connected successfully!
z3950> find title = rust
Records: 5
Result set: default
z3950> show 1 3
=== Record 1 ===
Leader: ...
  001: 123456
  245 10: $aRust Programming Language
...
z3950> scan rust
  "rust" (5)
  "rust programming" (3)
  "rust language" (2)
z3950> quit
```

---

## Error Handling

All client methods return `z3950_rs::Result<T>`, where errors are:

| Variant | Cause |
|---|---|
| `Error::Io` | TCP I/O failure |
| `Error::BerEncode` | Failed to encode a PDU |
| `Error::BerDecode` | Failed to decode a PDU |
| `Error::Protocol` | Unexpected PDU or closed connection |
| `Error::Marc` | MARC parsing failure |
| `Error::FrameTooLarge` | PDU exceeds the 16 MiB frame limit |
| `Error::InvalidOid` | Invalid ASN.1 Object Identifier |
| `Error::InvalidVisibleString` | Invalid visible string in PDU |
