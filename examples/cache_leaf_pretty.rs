//! A trivial example demonstrating the use of [CacheLeaf::read_file] with
//! CacheFieldCollection

use ccache_stats_reader::{CacheFieldCollection, CacheLeaf};
use std::path::PathBuf;

fn main() {
    let leaf = CacheLeaf::read_file(PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Must pass a path to a ccache 'stats' file"),
    ))
    .unwrap();
    leaf.write_pretty(std::io::stdout()).unwrap();
}
