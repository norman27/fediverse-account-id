use std::{error::Error};
//use std::error;
use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
//use serde_json::{Value};

// kann man nutzen anstatt id:0 bei keinen resultaten zurück zu geben
/*#[derive(Debug, Clone)]
struct EmptyResults;

impl fmt::Display for EmptyResults {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}
impl error::Error for EmptyResults {}*/

/// Simple program to resolve account id of a fediverse handle
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// Handle of the account to fetch the id, ie. @handle@node.domain
   #[arg(short, long)]
   name: String,
}

#[derive(Serialize, Deserialize)]
struct SearchResults {
    accounts: Vec<MastodonAccount>,
}

#[derive(Serialize, Deserialize)]
struct MastodonAccount {
    id: String,
    username: String,
}

pub struct Account {
    pub handle: String,
    pub node_domain: String,
}

pub fn parse_account(raw_account: String) -> Option<Account> {
    let re = Regex::new(r#"(?x)
        ^@{0,1}([\w]+)
        @([\w]+.[\w]+)$
        "#).unwrap();

    for cap in re.captures_iter(raw_account.as_str()) {
        return Some(Account { handle: cap[1].to_string(), node_domain: cap[2].to_string() })
    }

    let re = Regex::new(r#"(?x)
        ^https{0,1}://([\w]+.[\w]+)
        /@([\w]+)$
        "#).unwrap();

    for cap in re.captures_iter(raw_account.as_str()) {
        return Some(Account { handle: cap[2].to_string(), node_domain: cap[1].to_string() })
    }

    None
}

pub fn get_account_id(acc: Account) -> Result<String, Box<dyn Error>> {
    let url = format!("https://{}/api/v2/search?q={}&limit=1&type=accounts", acc.node_domain, acc.handle);

    let body = reqwest::blocking::get(url)?
    .text()?;

    let search_results: SearchResults = serde_json::from_str(body.as_str())?;

    match search_results.accounts.len() {
        0 => return Ok("0".to_string()),
        _ => {
            let ids = search_results.accounts
                .into_iter()
                .filter(|m_acc| m_acc.username.to_lowercase() == acc.handle.to_lowercase())
                .map(|m_acc| m_acc.id)
                .collect::<String>();
                
            return Ok(ids.to_string())
        },
    }
}

fn main() {
    let args = Args::parse();

    let acc = parse_account(args.name);

    match acc {
        Some(acc) => {
            let id = get_account_id(acc);
            match id.ok() {
                Some(id) => println!("ID: {:?}", id),
                None => println!("error querying id")
            };
        },
        None => println!("failed to parse account")
    }
}
