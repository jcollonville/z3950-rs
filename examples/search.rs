use std::convert::TryInto;
use std::env;

use z3950_rs::Client;

fn label(record: &z3950_rs::MarcRecord) -> String {
    if let Some(cf) = record.control_fields.iter().find(|f| f.tag == "001") {
        return format!("ID {}", cf.value);
    }
    if let Some(df) = record.data_fields.first() {
        return format!("{} {}", df.tag, df.ind1);
    }
    "<record>".to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut host = String::from("localhost");
    let mut port = 9999u16;
    let mut db = String::from("Default");
    let mut query = String::from("athena");
    let mut user: Option<String> = Some(String::from("Z3950"));
    let mut password: Option<String> = Some(String::from("Z3950_BNF"));

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--host" => {
                if i + 1 < args.len() {
                    host = args[i + 1].clone();
                    i += 1;
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().map_err(|e| format!("invalid --port value '{}': {e}", args[i + 1]))?;
                    i += 1;
                }
            }
            "--db" => {
                if i + 1 < args.len() {
                    db = args[i + 1].clone();
                    i += 1;
                }
            }
            "--query" => {
                if i + 1 < args.len() {
                    query = args[i + 1].clone();
                    i += 1;
                }
            }
            "--user" => {
                if i + 1 < args.len() {
                    user = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--password" => {
                if i + 1 < args.len() {
                    password = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let addr = format!("{}:{}", host, port);
    let creds = match (user.as_ref(), password.as_ref()) {
        (Some(u), Some(p)) => Some((u.as_str(), p.as_str())),
        _ => None,
    };
    let mut client = Client::connect_with_credentials(&addr, creds).await?;
    let search = client.search(&[db.as_str()], &query).await?;

    let result_count = search.result_count;
    let count: i64 = result_count.try_into().map_err(|_| "result_count too large for i64")?;
    let records = client.present_marc(1, count).await?;
    for (idx, r) in records.iter().enumerate() {
        println!("{:?}", r);
        println!("{} - {}", idx + 1, label(r));

    }

    Ok(())
}
