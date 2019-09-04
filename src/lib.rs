#![cfg_attr(feature = "external_doc", feature(external_doc))]
#![cfg_attr(feature = "non_exhaustive", feature(non_exhaustive))]
#![cfg_attr(feature = "external_doc", doc(include = "lib.md"))]
#![cfg_attr(
    not(feature = "external_doc"),
    doc = "This crate implements a simple interface for accessing `ccache` \
           stats without needing an `exec` call."
)]

mod cache_field;
pub use cache_field::{
    CacheField, CacheFieldData, CacheFieldFormat, CacheFieldMeta,
    FIELD_DATA_ORDER, FIELD_DISPLAY_ORDER,
};

#[cfg_attr(feature = "external_doc", doc(include = "ErrorKind.md"))]
#[cfg_attr(
    not(feature = "external_doc"),
    doc = "An enum for wrapping various errors emitted by this crate."
)]
#[cfg_attr(feature = "non_exhaustive", non_exhaustive)]
#[derive(Debug)]
pub enum ErrorKind {
    /// Proxy Enum for [std::io::Error]
    IoError(std::io::Error),
    /// Proxy Enum for internal errors that are simple [String]'s
    Stringy(String),
    /// Proxy enum for errors that are [std::time::SystemTimeError]'s
    SysTime(std::time::SystemTimeError),
}

impl From<std::io::Error> for ErrorKind {
    fn from(e: std::io::Error) -> Self { ErrorKind::IoError(e) }
}
impl From<std::time::SystemTimeError> for ErrorKind {
    fn from(e: std::time::SystemTimeError) -> Self { ErrorKind::SysTime(e) }
}

use chrono::{TimeZone, Utc};

#[cfg_attr(feature = "external_doc", doc(include = "CacheLeaf.md"))]
#[cfg_attr(
    not(feature = "external_doc"),
    doc = "A leaf container for one sub-cache of a ccache directory."
)]
#[derive(Debug, Clone, Copy)]
pub struct CacheLeaf {
    fields: CacheFieldData,
    mtime:  chrono::DateTime<Utc>,
}

impl Default for CacheLeaf {
    fn default() -> Self {
        Self { fields: Default::default(), mtime: Utc.timestamp(0, 0) }
    }
}

use std::{fs::File, io::Read, path::PathBuf};

impl CacheLeaf {
    /// Construct a [CacheLeaf] by reading a specified input file
    ///
    /// ```no_run
    /// use ccache_stats_reader::CacheLeaf;
    /// use std::path::PathBuf;
    /// let leaf = CacheLeaf::read_file(PathBuf::from("/var/tmp/ccache/0/stats"));
    /// ```
    pub fn read_file(f: PathBuf) -> Result<Self, ErrorKind> {
        let mut me: Self = Default::default();
        let mut my_file = File::open(&f)?;

        // This is a clusterfuck really, the internal .modified takes a lot of
        // mangling to get the internal unix-time value out of the metadata,
        // and when it does, its a u64, but chrono's timestamp takes an i64
        // ... there surely has to be a better way, but everything I tried at
        // least required me to rely on features that are very new in rust.
        me.mtime = Utc.timestamp(
            my_file
                .metadata()?
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64,
            0,
        );

        // We would use a BufReader here, but that cocks up amazingly when
        // some idiot passes a directory as the PathBuf, and BufReader
        // repeatedly invokes File::read() which repeatedly returns an
        // Err(), and as an Err() is a Some(Err()) not a None(),
        // doesn't end the iteration, and so on the next iteration ... it
        // calls read() again, gets the same result, and subsequently
        // iterates forever doing nothing.
        //
        // We have a <1k file, who cares!?
        let mut buf = String::new();
        my_file.read_to_string(&mut buf)?;
        // We collect all lines verbatim, and then use the FIELD_DATA_ORDER
        // array to pick values out of it. That way if there are lines in the
        // input source that we haven't coded behaviour for yet, it won't
        // result in array-index-out-of-bounds problems at runtime.
        //
        // The input source having fewer items than FIELD_DATA_ORDER is gated
        // by the field_addr <= last_line control, so too-few lines will
        // result in just a bunch of 0 entries in the dataset.
        let lines: Vec<&str> = buf.lines().collect();
        let last_line = lines.len() - 1;

        for field in FIELD_DATA_ORDER {
            let field_addr: usize = field.as_usize();
            if field_addr <= last_line {
                let line = &lines[field_addr];
                if let Ok(v) = line.parse::<u64>() {
                    me.fields.set_field(*field, v);
                } else {
                    unimplemented!(
                        "Line {} in {:?} did not parse as u64",
                        field_addr,
                        f
                    );
                }
            }
        }
        Ok(me)
    }
}

#[cfg_attr(feature = "external_doc", doc(include = "CacheDir.md"))]
#[cfg_attr(
    not(feature = "external_doc"),
    doc = "A container for collecting statistics from a ccache directory."
)]
#[derive(Debug, Clone, Copy)]
pub struct CacheDir {
    fields: CacheFieldData,
    mtime:  chrono::DateTime<Utc>,
}

impl Default for CacheDir {
    fn default() -> Self {
        Self { fields: Default::default(), mtime: Utc.timestamp(0, 0) }
    }
}

impl CacheDir {
    /// Read a specified ccache root directory and collect statistics
    pub fn read_dir(d: PathBuf) -> Result<Self, ErrorKind> {
        let mut me: Self = Default::default();
        me.add_leaf(d.join("stats"))?;
        for i in 0..=0xF {
            if let Some(c) = std::char::from_digit(i, 16) {
                me.add_leaf(d.join(c.to_string()).join("stats"))?;
            }
        }
        Ok(me)
    }

    fn stash_field(&mut self, field: CacheField, value: u64) {
        let current_value = self.fields.get_field(field);
        match field {
            CacheField::ZeroTimeStamp => {
                if value > current_value {
                    self.fields.set_field(field, value);
                }
            },
            _ => {
                self.fields.set_field(field, current_value + value);
            },
        }
    }

    fn add_leaf(&mut self, f: PathBuf) -> Result<(), ErrorKind> {
        let leaf_result = CacheLeaf::read_file(f);
        if let Ok(leaf) = &leaf_result {
            self.merge_leaf(leaf);
            return Ok(());
        }
        if let Err(e) = leaf_result {
            if let ErrorKind::IoError(io) = &e {
                if io.kind() == std::io::ErrorKind::NotFound {
                    return Ok(());
                }
            }
            return Err(e);
        }
        Ok(())
    }

    fn merge_leaf(&mut self, leaf: &CacheLeaf) {
        for field in FIELD_DATA_ORDER {
            let value = leaf.fields.get_field(*field);
            self.stash_field(*field, value);
        }
        if self.mtime < leaf.mtime {
            self.mtime = leaf.mtime;
        }
    }
}

#[cfg_attr(
    feature = "external_doc",
    doc(include = "CacheFieldCollection.md")
)]
#[cfg_attr(
    not(feature = "external_doc"),
    doc = "An abstact representation of 'a thing' that can expose data \
           about its unit."
)]
pub trait CacheFieldCollection {
    /// Returns a struct containing data recorded for this thing
    fn fields(&self) -> &CacheFieldData;
    /// Returns a timestamp indicating time of last modification/update
    fn mtime(&self) -> &chrono::DateTime<Utc>;
}

impl CacheFieldCollection for CacheLeaf {
    fn fields(&self) -> &CacheFieldData { &self.fields }

    fn mtime(&self) -> &chrono::DateTime<Utc> { &self.mtime }
}

impl CacheFieldCollection for CacheDir {
    fn fields(&self) -> &CacheFieldData { &self.fields }

    fn mtime(&self) -> &chrono::DateTime<Utc> { &self.mtime }
}
