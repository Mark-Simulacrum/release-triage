A collection of tools useful for triaging Rust.

Currently contains:

reverse-deps search of crates.io index, which prints out the amount of possible
reverse dependencies for a given crate. Example output is below.

```
$ cargo run --release --bin reverse-deps -- "petgraph 0.4"
    Finished release [optimized] target(s) in 0.0 secs
     Running `target/release/reverse-deps 'petgraph 0.4'`
fatal: destination path 'crates.io-index' already exists and is not an empty directory.
Already up-to-date.
loaded 11822 unique crates and 67275 versions
Created graph with 66839 nodes and 3648715 edges.
dependents on petgraph 0.4.0: 21 crates, 103 versions
dependents on petgraph 0.4.1: 22 crates, 108 versions
dependents on petgraph 0.4.2: 23 crates, 113 versions
dependents on petgraph 0.4.3: 24 crates, 117 versions
dependents on petgraph 0.4.4: 58 crates, 209 versions
dependents on petgraph 0.4.5: 154 crates, 562 versions
dependents on petgraph 0.4.6: 154 crates, 562 versions
dependents on petgraph 0.4.7: 155 crates, 565 versions
dependents on petgraph 0.4.8: 155 crates, 565 versions
dependents on petgraph 0.4.9: 155 crates, 565 versions
dependents on petgraph 0.4.10: 155 crates, 565 versions
total versions broken: 575 (0.86%)
total crates broken: 155 (1.31%)```
