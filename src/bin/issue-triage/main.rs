#![recursion_limit = "1024"]
#![feature(conservative_impl_trait)]

extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate futures;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
#[macro_use] extern crate log;
#[macro_use] extern crate error_chain;
extern crate env_logger;
extern crate csv;
extern crate chrono;
extern crate dotenv;
extern crate reqwest;

#[allow(warnings)]
mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Json(::serde_json::Error);
            Csv(::csv::Error);
        }
    }
}

use errors::*;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::BufReader;
use std::io::Read;
use chrono::{Date, Datelike, DateTime, Utc};
use dotenv::dotenv;

quick_main!(run);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Issue {
    pub pull_request: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

mod github;

fn run() -> Result<()> {
    env_logger::init().unwrap();
    dotenv().ok();

    debug!("Loading all issues");
    let issues: Vec<Issue> = match ::std::fs::File::open("cache.json") {
        Ok(i) => {
            let mut input = Vec::new();
            BufReader::new(i).read_to_end(&mut input).unwrap();
            serde_json::from_slice(&input)?
        },
        Err(_) => {
            let issues = github::get_issues()?;
            let file = ::std::fs::File::create("cache.json")?;
            serde_json::to_writer(file, &issues)?;
            issues.into_iter()
                .map(|v| Ok(serde_json::from_value(v)?)).collect::<Result<Vec<_>>>()?
        }
    };

    let mut buckets: BTreeMap<Date<Utc>, ((u64, u64, u64, u64), (u64, u64, u64, u64))> = BTreeMap::new();

    fn month(d: DateTime<Utc>) -> Date<Utc> {
        d.date().with_day(1).unwrap()
    }

    for issue in issues {
        if issue.pull_request.is_some() {
            (buckets.entry(month(issue.created_at)).or_insert_with(Default::default).1).1 += 1;

            if let Some(c) = issue.closed_at {
                (buckets.entry(month(c)).or_insert_with(Default::default).1).2 += 1;
            } else {
                (buckets.entry(month(issue.updated_at)).or_insert_with(Default::default).1).0 += 1;
            }
            continue;
        }

        (buckets.entry(month(issue.created_at)).or_insert_with(Default::default).0).1 += 1;

        if let Some(c) = issue.closed_at {
            (buckets.entry(month(c)).or_insert_with(Default::default).0).2 += 1;
        } else {
            (buckets.entry(month(issue.updated_at)).or_insert_with(Default::default).0).0 += 1;
        }
    }

    println!("month updated created closed total updated created closed total_pr");
    let mut total = 0;
    let mut total_pr = 0;
    for (month, values) in buckets {
        total += (values.0).1;
        total -= (values.0).2;
        total_pr += (values.1).1;
        total_pr -= (values.1).2;
        println!("{} {} {} {} {} {} {} {} {}", month.format("%Y-%m").to_string(),
            (values.0).0, (values.0).1, (values.0).2, total,
            (values.1).0, (values.1).1, (values.1).2, total_pr);
    }

    Ok(())
}
