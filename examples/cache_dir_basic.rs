//! A trivial example demonstrating the use of [CacheDir::read_file]

use ccache_stats_reader::{CacheDir, CacheFieldCollection, FIELD_DATA_ORDER};
use std::path::PathBuf;

fn main() {
    let leaf = CacheDir::read_dir(PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("Must pass a path to a ccache root dir"),
    ))
    .unwrap();

    let data = leaf.fields();

    for &field in FIELD_DATA_ORDER {
        println!("{:?}: {}", field, data.get_field(field));
    }
}
