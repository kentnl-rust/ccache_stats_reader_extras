//! A trivial example demonstrating the use of [CacheLeaf::read_file] with
//! CacheFieldPrettyPrinter

use ccache_stats_reader::{CacheFieldPrettyPrinter, CacheLeaf};
use std::path::PathBuf;

fn main() {
    let leaf = CacheLeaf::read_file(PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Must pass a path to a ccache 'stats' file"),
    ))
    .unwrap();
    leaf.pretty_print();
}
