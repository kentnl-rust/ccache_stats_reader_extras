//! A trivial example demonstrating the use of [CacheDir::read_file] with
//! [CacheFieldCollection]

use ccache_stats_reader::{CacheDir, CacheFieldCollection};
use std::path::PathBuf;

fn main() {
    let leaf = CacheDir::read_dir(PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Must pass a path to a ccache root dir"),
    ))
    .unwrap();
    leaf.write_raw(std::io::stdout()).unwrap();
}
