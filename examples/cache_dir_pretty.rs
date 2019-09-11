//! A trivial example demonstrating the use of [CacheDir::read_file] with
//! [CacheFieldPrettyPrinter]

use ccache_stats_reader::{CacheDir, CacheFieldPrettyPrinter};
use std::path::PathBuf;

fn main() {
    let leaf = CacheDir::read_dir(PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Must pass a path to a ccache root dir"),
    ))
    .unwrap();
    leaf.pretty_print();
}
