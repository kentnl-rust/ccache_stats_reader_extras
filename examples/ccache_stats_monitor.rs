//! A statistics monitor for ccache
use ccache_stats_reader::{CacheDir, CacheFieldCollection};
use std::{env, path::PathBuf, thread, time};

fn main() {
    let cache_path = cache_path();
    let mut cd = CacheDir::read_dir(&cache_path).unwrap();
    loop {
        let elapsed = sleep_upto(5_000);
        println!("== {:?} ==", time::Instant::now());
        let new_cd = CacheDir::read_dir(&cache_path).unwrap();
        for (field, value) in new_cd.iter() {
            if field.metadata().is_flag_never() {
                continue;
            }
            let old = cd.get_field(field);
            if value != old {
                let diff = value - old;
                println!(
                    "{:?} {} -> {} ( ~{} @ {:.3}/sec )",
                    field,
                    old,
                    value,
                    diff,
                    (diff as f64) / elapsed
                );
            }
        }
        cd = new_cd;
    }
}

fn sleep_upto(t: u64) -> f64 {
    let poll_duration = time::Duration::from_millis(t / 10);
    let duration = time::Duration::from_millis(t);
    let now = time::Instant::now();
    loop {
        thread::sleep(poll_duration);
        if now.elapsed() > duration {
            break;
        }
    }
    let e = now.elapsed();
    (e.as_secs() as f64) + ((e.subsec_millis() as f64) / 1000.0)
}

fn cache_path() -> PathBuf {
    match env::var_os("CCACHE_DIR") {
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
    }
}
