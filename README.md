A collection of tools useful for triaging Rust.

Currently contains:

reverse-deps search of crates.io index, which prints out the amount of possible
reverse dependencies for a given crate. Example output is below.

```
$ cargo run --release --bin reverse-deps -- "petgraph 0.4"
   Compiling release-triage v0.1.0
    Finished release [optimized] target(s) in 1.63 secs
     Running `target/release/reverse-deps 'petgraph 0.4'`
fatal: destination path 'crates.io-index' already exists and is not an empty directory.
Already up-to-date.
loaded 11822 unique crates and 67273 versions
Created graph with 66837 nodes and 3648646 edges.
dependents on petgraph 0.4.0: 102
dependents on petgraph 0.4.1: 107
dependents on petgraph 0.4.2: 112
dependents on petgraph 0.4.3: 116
dependents on petgraph 0.4.4: 208
dependents on petgraph 0.4.5: 561
dependents on petgraph 0.4.6: 561
dependents on petgraph 0.4.7: 564
dependents on petgraph 0.4.8: 564
dependents on petgraph 0.4.9: 564
dependents on petgraph 0.4.10: 564
total broken: 4023 (6.02%)
```
