//! A trivial example demonstrating the use of [CacheLeaf::read_file]

use ccache_stats_reader::{CacheFieldCollection, CacheLeaf};
use std::path::PathBuf;

fn main() {
    let leaf = CacheLeaf::read_file(PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Must pass a path to a ccache 'stats' file"),
    ))
    .unwrap();

    for (field, value) in leaf.iter() {
        println!("{:?}: {}", field, value);
    }
}
