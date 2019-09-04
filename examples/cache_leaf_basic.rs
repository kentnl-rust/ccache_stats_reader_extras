//! A trivial example demonstrating the use of [CacheLeaf::read_file]

use ccache_stats_reader::CacheLeaf;
use std::path::PathBuf;

fn main() {
    let _leaf = CacheLeaf::read_file(PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Must pass a path to a ccache 'stats' file"),
    ))
    .unwrap();
}
