extern crate semver;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate petgraph;

use std::process::Command;
use std::path::Path;
use std::fs::{self, DirEntry, File};
use std::io::{self, Read};
use std::collections::{HashMap, HashSet};
use std::cmp::{PartialEq, Eq, PartialOrd, Ord, Ordering};
use std::env;
use std::fmt;

use semver::{Version, VersionReq};
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

    let count = map.values().map(|v| v.len()).sum::<usize>();

    println!("loaded {} unique crates and {} versions",
        map.len(), count);

    let mut nodes = HashMap::<CrateId, _>::with_capacity(count);
    let mut graph: Graph<CrateId, ()> = Graph::with_capacity(count, count);

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
        roots.extend(nodes.keys().filter(|k| k.to_string().starts_with(&arg)));
    }
    roots.sort();

    let mut total_broken = HashSet::new();
    for root in roots {
        let root_node = nodes[&*root];
        let mut processed = HashSet::new();
        let mut to_process = vec![root_node];
        while let Some(node) = to_process.pop() {
            processed.insert(node);
            // What will break if this node breaks: the incoming edges are from
            // crates that depend on us.
            for edge in graph.edges_directed(node, Direction::Incoming) {
                if !processed.contains(&edge.source()) {
                    to_process.push(edge.source());
                }
            }
        }

        processed.remove(&root_node);
        total_broken.extend(processed.clone());

        let mut dependents = processed.into_iter().map(|p| &graph[p]).collect::<Vec<_>>();
        dependents.sort();
        let dependents = dependents.into_iter().map(|p| p.to_string()).collect::<Vec<_>>();

        if dependents.len() < 20 {
            println!("dependents on {}: {}: {:#?}", root, dependents.len(), dependents);
        } else {
            println!("dependents on {}: {}", root, dependents.len());
        }
    }

    println!("total broken: {} ({:.2}%)",
        total_broken.len(),
        ((total_broken.len() as f64) / (graph.raw_nodes().len() as f64)) * 100.0);
}

#[derive(Debug, PartialEq, Deserialize)]
struct Dependency {
    name: String,
    req: VersionReq,
}

#[derive(Debug, Hash, PartialEq, PartialOrd, Ord, Eq)]
struct CrateId {
    name: String,
    version: Version
}

#[derive(Deserialize, Debug)]
struct Crate {
    name: String,
    #[serde(rename="vers")]
    version: Version,
    #[serde(rename="deps")]
    dependencies: Vec<Dependency>,
}

impl Crate {
    fn id(&self) -> CrateId {
        CrateId {
            name: self.name.clone(),
            version: self.version.clone(),
        }
    }
}

impl fmt::Display for CrateId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}.{}.{}",
            self.name, self.version.major, self.version.minor, self.version.patch)?;
        if !self.version.pre.is_empty() {
            write!(f, "-")?;
        }
        for pre in &self.version.pre {
            write!(f, "{}", pre)?;
        }
        if !self.version.build.is_empty() {
            write!(f, "+")?;
        }
        for build in &self.version.build {
            write!(f, "{}", build)?;
        }
        Ok(())
    }
}

impl PartialOrd for Crate {
    fn partial_cmp(&self, other: &Crate) -> Option<Ordering> {
        Some(self.name.cmp(&other.name).then_with(|| self.version.cmp(&other.version)))
    }
}

impl Ord for Crate {
    fn cmp(&self, other: &Crate) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Crate {
    fn eq(&self, other: &Crate) -> bool {
        self.name == other.name && self.version == other.version
    }
}

impl Eq for Crate {}

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
