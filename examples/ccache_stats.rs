//! A workalike stats-only version of ccache
use ccache_stats_reader::{CacheDir, CacheFieldCollection};
use std::{env, path::PathBuf};

enum Mode {
    Pretty,
    Raw,
}

fn main() {
    let args = std::env::args().skip(1);
    let mut mode = Mode::Pretty;
    for arg in args {
        match arg.as_str() {
            "-s" | "--show-stats" => {
                mode = Mode::Pretty;
            },
            "--print-stats" => {
                mode = Mode::Raw;
            },
            "-h" | "--help" => {
                println!(
                    "\
Usage:
    ccache_stats [options]
Options:
    -s, --show-stats    show summary of statistics counters
                        in human-readable format.
    --print-stats       print statistics counter IDs and 
                        corresponding values in machine-parsable format
    -h, --help          show this help message"
                );
                return;
            },
            e => {
                panic!("Unknown argument {}", e);
            },
        }
    }
    let cache_dir = match env::var_os("CCACHE_DIR") {
        Some(d) => match d.into_string() {
            Ok(v) => PathBuf::from(v),
            Err(e) => panic!("Could not parse: {:?}", e),
        },
        None => match env::var_os("HOME") {
            Some(d) => match d.into_string() {
                Ok(v) => PathBuf::from(v).join(".ccache"),
                Err(e) => panic!("Could not decode HOME: {:?}", e),
            },
            None => {
                panic!("Could not determine CCACHE_DIR: Not set, no HOME set")
            },
        },
    };
    let cd = CacheDir::read_dir(cache_dir).unwrap();
    match mode {
        Mode::Pretty => cd.write_pretty(std::io::stdout()).unwrap(),
        Mode::Raw => cd.write_raw(std::io::stdout()).unwrap(),
    };
}
