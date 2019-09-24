#![doc(html_root_url = "https://docs.rs/ccache_stats_reader/0.1.1")]
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
};
use cache_field::{FIELD_DATA_ORDER, FIELD_DISPLAY_ORDER};

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
    /// A path to a non-file was passed to CacheLeaf for reading,
    /// but it turned out to be the kind of thing that can't be expected to
    /// be read (like a directory)
    CacheLeafNonFile {
        /// The Path
        input_path: PathBuf,
    },
}

impl From<std::io::Error> for ErrorKind {
    fn from(e: std::io::Error) -> Self { ErrorKind::IoError(e) }
}
impl From<std::time::SystemTimeError> for ErrorKind {
    fn from(e: std::time::SystemTimeError) -> Self { ErrorKind::SysTime(e) }
}
impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::IoError(e) => e.fmt(f),
            ErrorKind::SysTime(e) => e.fmt(f),
            ErrorKind::Stringy(s) => write!(f, "{}", s),
            ErrorKind::ParseU64Error {
                input_value,
                input_file,
                input_line,
            } => write!(
                f,
                "could not parse u64 from value {:?} in {:?} line {}",
                input_value, input_file, input_line
            ),
            ErrorKind::CacheLeafNonFile { input_path } => write!(
                f,
                "expected path {:?} to be a readable file, not a directory",
                input_path
            ),
        }
    }
}
impl std::error::Error for ErrorKind {}

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
        let my_meta = my_file.metadata()?;

        // Metadata.is_file() only asserts the inode(7) type is a S_IFREG,
        // which excludes various classes of file descriptors that are
        // openable and readable in normal conditions, for instance,
        // S_IFIFO, which one could trip into using if they invoked the
        // command in a shell, and used shell redirection to simulate
        // a file, eg:
        //
        // ccache_stats_leaf <( program_generates_a_stats_file_to_stdout )
        //
        // This passes (on unix) a pipe such as /dev/fd/63 such that:
        //    ( st_mode & S_IFMT ) == S_IFIFO
        //
        // (Where: S_IFMT = 0_170_00, S_IFIFO = 0_010_000)
        //
        // Demo:
        //  perl -e 'my ($dev, $ino, $mode, @rest) = stat($ARGV[0]);
        //           printf qq[%s => %07O\n], $ARGV[0], $mode;
        //           printf qq[%07O\n], $mode & 0_170_000 ' <( echo foo )
        //  /dev/fd/63 => 0010600
        //  0010000
        //
        // using is_file() here would cause it to bail, when continuing is
        // just fine.
        if my_meta.is_dir() {
            return Err(ErrorKind::CacheLeafNonFile { input_path: f });
        }
        // This is a clusterfuck really, the internal .modified takes a lot of
        // mangling to get the internal unix-time value out of the metadata,
        // and when it does, its a u64, but chrono's timestamp takes an i64
        // ... there surely has to be a better way, but everything I tried at
        // least required me to rely on features that are very new in rust.
        me.mtime = Utc.timestamp(
            // Returns a timestamp indicating time of last
            // modification/update
            my_meta
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
    pub fn read_dir<P>(d: P) -> Result<Self, ErrorKind>
    where
        P: Into<PathBuf>,
    {
        let mut me: Self = Default::default();
        let dir: PathBuf = d.into();
        me.add_leaf(dir.join("stats"))?;
        for i in 0..=0xF {
            if let Some(c) = std::char::from_digit(i, 16) {
                me.add_leaf(dir.join(c.to_string()).join("stats"))?;
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

use chrono::Local;

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
    /// Returns a value for the named field
    fn get_field(&self, f: CacheField) -> u64 { self.fields().get_field(f) }
    /// Returns an iterator of (field, value) pairs in display order to
    /// simplify loop and chain flow controls
    fn iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = (CacheField, u64)> + 'a> {
        Box::new(
            FIELD_DISPLAY_ORDER
                .iter()
                .map(move |&field| (field, self.get_field(field).to_owned())),
        )
    }
    /// Writes the data in this collection to the designated destination (such
    /// as [std::io::stdout]) in a format similar to that produced by
    /// `ccache --print-stats`
    fn write_raw(
        &self, mut fh: impl std::io::Write,
    ) -> Result<(), ErrorKind> {
        let mtime = self.mtime();
        writeln!(fh, "stats_updated_timestamp\t{}", mtime.timestamp())?;
        for (field, value) in self.iter() {
            if field.metadata().is_flag_never() {
                continue;
            }
            writeln!(fh, "{}\t{}", field.metadata().id, value)?;
        }
        Ok(())
    }

    /// Writes the data in this collection to the designated destination (such
    /// as [std::io::stdout]) in a format similar to that produced by
    /// `ccache -s`
    fn write_pretty(
        &self, mut fh: impl std::io::Write,
    ) -> Result<(), ErrorKind> {
        let mtime = self.mtime();
        writeln!(
            fh,
            "{:<30} {:>9}",
            "stats updated",
            Local.timestamp(mtime.timestamp(), 0),
        )?;
        for (field, value) in self.iter() {
            let meta = field.metadata();
            if meta.is_flag_never() {
                continue;
            }
            if !meta.is_flag_always() && value == 0u64 {
                continue;
            }
            writeln!(
                fh,
                "{:<30} {:>9}",
                field.metadata().message,
                field.format_value(value)
            )?;
        }
        Ok(())
    }
}

impl CacheFieldCollection for CacheLeaf {
    fn fields(&self) -> &CacheFieldData { &self.fields }

    fn mtime(&self) -> &chrono::DateTime<Utc> { &self.mtime }
}

impl CacheFieldCollection for CacheDir {
    fn fields(&self) -> &CacheFieldData { &self.fields }

    fn mtime(&self) -> &chrono::DateTime<Utc> { &self.mtime }
}
