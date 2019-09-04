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
    /// Error parsing a u64 from a file
    ParseU64Error {
        /// The string that was attempted to be parsed
        input_value: String,
        /// The line of the file that was trying to be decoded
        input_line: usize,
        /// The file that was being read
        input_file: PathBuf,
    },
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

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

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
        let my_file = File::open(&f)?;

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

        // Note the default of 8k for BufReader is excessive for us, as it
        // accounts for 8/9ths of the overall heap size, which is
        // silly when you consider the file we're reading is typically
        // under 200 *bytes*, and all lines are under *21* bytes each,
        // and the whole point of using BufReader is to get the read_line()
        // abstraction.
        let mut bufreader = BufReader::with_capacity(100, my_file);
        let mut buf = String::new();

        for field in FIELD_DATA_ORDER {
            // We have to use this readline + match pattern, because the
            // default implementation of BufReader() + lines().collect() fails
            // abysmally if a user passes a directory instead of a file, as
            // the implementation of the Lines iterator has no state, and just
            // relays any errors it gets as a Some(Err()).
            //
            // So the collect() calls next() repeatedly, each time getting the
            // same Some(Err()), and never returning a None(), even though
            // read() on a directory will never return anything other than an
            // Err().
            //
            // So the iterator generates an infinite stream of Some(Err())
            // which collect then tries to make a vector from, and that turns
            // out to be a good way to eat ram and do nothing else.
            //
            // Bug: https://github.com/rust-lang/rust/issues/64144
            //
            match bufreader.read_line(&mut buf) {
                // If we run out of input lines before we reach the end of
                // FIELD_DATA_ORDER, remaining fields will be
                // their default (0) and we just stop reading the file.
                Ok(0) => break,
                // This fork should be followed on the first read_line call if
                // the fh represents a directory.
                // Otherwise it will be followed only on file read errors
                Err(e) => return Err(ErrorKind::IoError(e)),
                Ok(_n) => {
                    // Cribbed from the BufRead source
                    // trims trailing linefeed tokens.
                    if buf.ends_with('\n') {
                        let _ = buf.pop();
                        if buf.ends_with('\r') {
                            let _ = buf.pop();
                        }
                    }
                    let field_addr: usize = field.as_usize();
                    if let Ok(v) = buf.parse::<u64>() {
                        me.fields.set_field(*field, v);
                    } else {
                        return Err(ErrorKind::ParseU64Error {
                            input_line:  field_addr,
                            input_value: buf,
                            input_file:  f,
                        });
                    }
                    // important: otherwise buf grows forever.
                    buf.clear();
                },
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
