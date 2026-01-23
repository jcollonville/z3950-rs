# z3950-rs

[![crates.io](https://img.shields.io/crates/v/z3950-rs.svg)](https://crates.io/crates/z3950-rs)
[![docs.rs](https://docs.rs/z3950-rs/badge.svg)](https://docs.rs/z3950-rs)

Minimal asynchronous (Tokio) Z39.50 client with MARC parsing via [`marc-rs`](https://crates.io/crates/marc-rs). PDUs are encoded/decoded in ASN.1 BER using [`rasn`](https://crates.io/crates/rasn).

## Key features
- Init connection
- Search/Find with type-1 query (BIB-1, title attribute)
- Present to retrieve MARC records (USMARC) and convert them to `marc_rs::Record`

## Status
Basic demonstration-oriented implementation. The `scan` operation is not yet implemented.

## Usage
```rust
use z3950_rs::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = Client::connect_with_credentials(
        "z3950.loc.gov:7090",
        Some(("user", "password")),
    )
    .await?;
    let _search = client.search(&["Voyager"], "rust").await?;
    let records = client.present_marc(1, 5).await?;

    for r in records {
        if let Some(title) = r.title() {
            println!("{}", title);
        }
    }
    Ok(())
}
```

## CLI Example
A simple example is available in `examples/search.rs`.

```
cargo run --example search -- --host z3950.loc.gov --port 7090 --db Voyager --query rust --user foo --password bar
```
