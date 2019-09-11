//! A trivial example demonstrating the use of [CacheLeaf::read_file] with
//! CacheFieldRawPrinter

use ccache_stats_reader::{CacheFieldRawPrinter, CacheLeaf};
use std::path::PathBuf;

fn main() {
    let leaf = CacheLeaf::read_file(PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Must pass a path to a ccache 'stats' file"),
    ))
    .unwrap();
    leaf.raw_print();
}
