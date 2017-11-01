extern crate semver;
extern crate serde_json;
extern crate serde;
extern crate petgraph;
extern crate release_triage;

use std::process::Command;
use std::path::Path;
use std::fs::{self, DirEntry, File};
use std::io::{self, Read};
use std::collections::{HashMap, HashSet};
use std::env;

use release_triage::crates::{Crate, CrateId};

use semver::VersionReq;
use petgraph::{Graph, Direction};
use petgraph::visit::EdgeRef;

fn run(dir: &str, s: &str) -> bool {
    let mut args = s.split(" ");
    Command::new(args.next().unwrap())
        .current_dir(dir)
        .args(args)
        .status().is_ok()
}

fn main() {
    run(".", "git clone https://github.com/rust-lang/crates.io-index crates.io-index");
    if !run("crates.io-index", "git pull") {
        eprintln!("failed to update index");
    }

    let map = get_all_crates();

    let version_count = map.values().map(|v| v.len()).sum::<usize>();
    let crate_count = map.len();

    println!("loaded {} unique crates and {} versions",
        crate_count, version_count);

    let mut nodes = HashMap::<CrateId, _>::with_capacity(version_count);
    let mut graph: Graph<CrateId, ()> = Graph::with_capacity(version_count, version_count);

    for krate in map.values().flat_map(|v| v) {
        let krate_node = *nodes.entry(krate.id())
            .or_insert_with(|| graph.add_node(krate.id()));
        for dependency in &krate.dependencies {
            let resolutions = map.get(&dependency.name)
                .or_else(|| map.get(&dependency.name.replace("_", "-")))
                .unwrap_or_else(|| {
                    panic!("could not find {}", dependency.name);
                })
                .iter().filter(|r| dependency.req.matches(&r.version));
            for resolution in resolutions {
                let dep_node = *nodes.entry(resolution.id())
                    .or_insert_with(|| graph.add_node(resolution.id()));
                graph.update_edge(krate_node, dep_node, ());
            }
        }
    }

    println!("Created graph with {} nodes and {} edges.",
        graph.raw_nodes().len(), graph.raw_edges().len());

    let mut roots = Vec::new();
    for arg in env::args().skip(1) {
        let space = arg.find(" ").unwrap_or(arg.len());
        let name = &arg[..space];
        let version = &arg[space..].trim();
        let version = VersionReq::parse(version).unwrap_or_else(|e| {
            panic!("{}: failed to parse version req {}: {:?}", name, version, e);
        });
        roots.extend(nodes.keys().filter(|k| {
            k.name == name && version.matches(&k.version)
        }));
    }
    roots.sort();

    let mut total_broken = HashSet::new();
    let mut total_crates = HashSet::new();
    for root in roots {
        let root_node = nodes[&*root];
        let mut versions = HashSet::new();
        let mut crate_names = HashSet::new();
        let mut to_process = vec![root_node];
        while let Some(node) = to_process.pop() {
            let crate_id = &graph[node];
            versions.insert(node);
            crate_names.insert(&crate_id.name);
            // What will break if this node breaks: the incoming edges are from
            // crates that depend on us.
            for edge in graph.edges_directed(node, Direction::Incoming) {
                if !versions.contains(&edge.source()) {
                    to_process.push(edge.source());
                }
            }
        }

        {
            let mut dependents = versions.iter().collect::<Vec<_>>();
            dependents.sort();
            let dependents = dependents.into_iter().map(|p| graph[*p].to_string()).collect::<Vec<_>>();

            if dependents.len() < 20 && env::var_os("QUIET").is_none() {
                println!("dependents on {}: {} crates, {} versions: {:#?}",
                    root, crate_names.len(), dependents.len(), dependents);
            } else {
                println!("dependents on {}: {} crates, {} versions",
                    root, crate_names.len(), dependents.len());
            }
        }

        total_broken.extend(versions);
        total_crates.extend(crate_names);
    }

    println!("total versions broken: {} ({:.2}%)",
        total_broken.len(),
        ((total_broken.len() as f64) / (graph.raw_nodes().len() as f64)) * 100.0);
    println!("total crates broken: {} ({:.2}%)",
        total_crates.len(),
        ((total_crates.len() as f64) / (crate_count as f64)) * 100.0);
}

// crate => crates which depend on the key
fn get_all_crates() -> HashMap<String, Vec<Crate>> {
    let mut map = HashMap::new();
    visit_dirs(Path::new("crates.io-index"), &mut |entry| {
        let name = entry.file_name();
        if name.to_string_lossy() == "config.json" { return; }
        for line in read_file(&entry.path()).lines() {
            let krate: Crate = serde_json::from_str(&line).unwrap_or_else(|e| {
                panic!("failed to parse {:?}: {:?}", entry.path(), e);
            });
            map.entry(krate.name.clone()).or_insert_with(Vec::new).push(krate);
        }
    }).unwrap();
    map
}

fn visit_dirs<F: FnMut(&DirEntry)>(dir: &Path, cb: &mut F) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if !entry.path().to_string_lossy().contains(".git") {
                    visit_dirs(&path, cb)?;
                }
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn read_file(path: &Path) -> String {
    let mut file = File::open(path).expect("opened file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap_or_else(|e| {
        panic!("failed to read {:?}: {:?}", path, e);
    });
    contents
}
