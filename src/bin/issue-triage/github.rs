//! Get commits through the Github API.

use std::str;
use std::env;

use serde_json::{self, Value};

use errors::Result;
use futures::{Future, Stream};
use hyper::header::{Link, RelationType, Authorization, UserAgent};
use hyper::Method;
use tokio_core::reactor::{Handle, Core};
use futures::sync::mpsc;
use futures::future;
use reqwest::unstable::async::{Client, Response};

fn token() -> String {
    env::var("GH_API_TOKEN").unwrap()
}

pub fn get_issues() -> Result<Vec<Value>> {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());
    let (sender, receiver) = mpsc::unbounded();

    fn fire_request(
        client: &Client,
        handle: Handle,
        url: &str,
        sender: mpsc::UnboundedSender<Response>,
    ) -> impl Future<Item = Response, Error = ::reqwest::Error> {
        info!("Requesting: {}", url);
        let mut req = client.request(Method::Get, url);
        req.header(Authorization(format!("token {}", token())));
        req.header(UserAgent::new("Issue-Investigator"));
        let url = url.to_string();
        let c = client.clone();
        req.send().map(move |res| {
            info!("{} => {}", url, res.status());
            res
        }).map(move |res| {
            if let Some(headers) = res.headers().get::<Link>() {
                if let Some(last) = headers.values().iter().find(|v| v.rel() == Some(&[RelationType::Last])) {
                    debug!("last page: {:?}", last.link());
                }
                if let Some(next) = headers.values().iter().find(|v| v.rel() == Some(&[RelationType::Next])) {
                    let r = fire_request(&c, handle.clone(), next.link(), sender.clone()).map(move |resp| {
                        sender.unbounded_send(resp).unwrap();
                    }).map_err(|e| {
                        info!("problem: {:?}", e);
                        ()
                    });
                    handle.spawn(r);
                }
            }
            res
        })
    }

    let s1 = sender.clone();
    let r = fire_request(
        &client,
        core.handle(),
        "https://api.github.com/repos/rust-lang/rust/issues?per_page=100&state=all&filter=all",
        sender.clone(),
    ).map(move |resp| {
        s1.unbounded_send(resp).unwrap();
    }).map_err(|e| {
        info!("problem: {:?}", e);
        ()
    });
    core.handle().spawn(r);

    let r2 = receiver.fold(Vec::<Value>::new(), move |mut acc, res| {
        if !res.status().is_success() {
            return future::Either::A(future::ok(acc));
        }

        future::Either::B(res.into_body().concat2().map(|body| {
            let body = str::from_utf8(&body).unwrap();
            let val: Vec<Value> = serde_json::from_str(&body).unwrap();
            info!("read {} issues", val.len());
            acc.extend(val);
            acc
        }).map_err(|e| {
            info!("problem: {:?}", e);
            ()
        }))
    });

    ::std::mem::drop(sender);

    Ok(core.run(r2).unwrap())
}
