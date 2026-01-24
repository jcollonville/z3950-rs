# z3950-rs

[![crates.io](https://img.shields.io/crates/v/z3950-rs.svg)](https://crates.io/crates/z3950-rs)
[![docs.rs](https://docs.rs/z3950-rs/badge.svg)](https://docs.rs/z3950-rs)

Minimal asynchronous (Tokio) Z39.50 client with MARC parsing via [`marc-rs`](https://crates.io/crates/marc-rs). PDUs are encoded/decoded in ASN.1 BER using [`rasn`](https://crates.io/crates/rasn).

## Supported Z39.50 Requests

The library supports the following Z39.50 operations:

- **Init** - Initialize connection with authentication support
- **Search** - Type-1 query (RPN) with BIB-1 attributes
- **Present** - Retrieve MARC records (USMARC) and convert them to `marc_rs::Record`
- **Scan** - Browse index terms
- **Sort** - Sort result sets
- **DeleteResultSet** - Delete one or all result sets
- **Close** - Close connection gracefully
- **ExtendedServices** - Extended services support
- **DuplicateDetection** - Duplicate detection operations
- **ResourceControl** - Resource control operations
- **ResourceReport** - Resource reporting
- **AccessControl** - Access control and authentication

## Usage

### Library API

```rust
use z3950_rs::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = Client::connect_with_credentials(
        "z3950.loc.gov:7090",
        Some(("user", "password")),
    )
    .await?;
    
    // Search
    let _search = client.search(&["Voyager"], "rust").await?;
    
    // Present records
    let records = client.present_marc(1, 5).await?;
    for r in records {
        if let Some(title) = r.title() {
            println!("{}", title);
        }
    }
    
    // Scan index
    let scan_response = client.scan(&["Voyager"], "athena", 4, 20, None).await?;
    
    Ok(())
}
```

### CLI Client

The library includes an interactive CLI client compatible with `yaz-client` commands.

#### Installation

```bash
cargo install z3950-rs --features cli
```

#### Basic Usage

```bash
# Connect and use interactively
z3950 -s localhost:9999 -d Default

# With authentication
z3950 -s localhost:9999 -d Default -u username -p password

# JSON output format
z3950 -s localhost:9999 -d Default --format json
```

#### Available Commands

Once connected, the following commands are available:

- **`open <host:port>`** - Connect to a Z39.50 server
- **`close`** - Close the current connection
- **`find <query>`** - Search for records (e.g., `find rust programming`)
- **`show [start] [count]`** - Display records (default: `show 1 10`)
- **`scan <term>`** - Browse index starting from a term
- **`set <option> <value>`** - Set options:
  - `set database <name>` - Set database name(s), comma-separated
  - `set result-set <name>` - Set result set name
  - `set format text|json` - Set output format
- **`get <option>`** - Get current option value (database, result-set, format)
- **`help`** or **`?`** - Show help
- **`quit`** or **`exit`** - Exit the client

#### Example Session

```
$ z3950 -s localhost:9999 -d Default
Connecting to localhost:9999...
Connected successfully!
z3950> find athena
Records: 5
Result set: default
z3950> show 1 3
=== Record 1 ===
Leader: ...
  001: 123456
  245 10: $aAthena
...
z3950> scan athena
  "athena" (10)
  "athena database" (5)
  "athena query" (3)
z3950> quit
```

## Examples

A simple example is available in `examples/search.rs`:

```bash
cargo run --example search -- --host z3950.loc.gov --port 7090 --db Voyager --query rust --user foo --password bar
```
