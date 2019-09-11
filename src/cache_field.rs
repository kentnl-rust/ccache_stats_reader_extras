#[cfg_attr(feature = "external_doc", doc(include = "CacheFieldFormat.md"))]
#[cfg_attr(
    not(feature = "external_doc"),
    doc = "A descriptor enum for which formatting to use for a given field."
)]
#[derive(Debug, Clone, Copy)]
pub enum CacheFieldFormat {
    /// No Special Formatting
    None,
    /// Format as a timestamp
    TimeStamp,
    /// Format as a byte-size
    SizeTimes1024,
}

const FLAG_NONE: u8 = 0;
const FLAG_NOZERO: u8 = 1;
const FLAG_ALWAYS: u8 = 2;
const FLAG_NEVER: u8 = 4;

#[cfg_attr(feature = "external_doc", doc(include = "CacheFieldMeta.md"))]
#[cfg_attr(
    not(feature = "external_doc"),
    doc = "A container for metadata about various fields."
)]
#[derive(Debug, Clone, Copy)]
pub struct CacheFieldMeta {
    pub(super) id:      &'static str,
    pub(super) message: &'static str,
    format:             CacheFieldFormat,
    flags:              u8,
}

impl CacheFieldMeta {
    pub(super) fn is_flag_always(&self) -> bool {
        self.flags & FLAG_ALWAYS == FLAG_ALWAYS
    }

    pub(super) fn is_flag_never(&self) -> bool {
        self.flags & FLAG_NEVER == FLAG_NEVER
    }
}

#[cfg_attr(feature = "external_doc", doc(include = "CacheField.md"))]
#[cfg_attr(
    not(feature = "external_doc"),
    doc = "An enum based field definition for ccache's data fields."
)]
#[cfg_attr(feature = "non_exhaustive", non_exhaustive)]
#[derive(Debug, Clone, Copy)]
// See ccache.h/.hpp
pub enum CacheField {
    /// Silly implementation detail
    None                 = 0,
    /// Counter of instances where the compiler produced stdout
    StdOut               = 1,
    /// Counter of compile failures
    Status               = 2,
    /// Counter of internal errors in ccache
    Error                = 3,
    /// Counter of cache misses
    ToCache              = 4,
    /// Counter of preprocessor errors
    PreProcessor         = 5,
    /// Counter of being unable to find the compiler
    Compiler             = 6,
    /// Counter of ccache being unable to find a file in cache
    Missing              = 7,
    /// Counter of pre-processed cache-hits
    CacheHitCpp          = 8,
    /// Counter of bad compiler arguments
    Args                 = 9,
    /// Counter of ccache being called for link
    Link                 = 10,
    /// Counter of the number of files in the cache
    NumFiles             = 11,
    /// Counter of the total size of the cache
    TotalSize            = 12,
    /// (Obsolete) Maximum files in cache
    ObsoleteMaxFiles     = 13,
    /// (Obsolete) Maximum size of cache
    ObsoleteMaxSize      = 14,
    /// Counts of being called with an unsupported source language
    SourceLang           = 15,
    /// Counts of being unable to write to output file
    BadOutputFile        = 16,
    /// Counts of being called without an input file
    NoInput              = 17,
    /// Counter of calls with multiple source files
    Multiple             = 18,
    /// Counter of autoconf compiles/links
    ConfTest             = 19,
    /// Counter of calling compiler with an unsupported option
    UnsupportedOption    = 20,
    /// Counter of output to stdout
    OutStdOut            = 21,
    /// Counter of direct cache hits
    CacheHitDir          = 22,
    /// Counter of compiler producing no output
    NoOutput             = 23,
    /// Counter of compiler producing empty output
    EmptyOutput          = 24,
    /// Counter of encountering an error hashing an extra file
    BadExtraFile         = 25,
    /// Counter of failed compiler checks
    CompCheck            = 26,
    /// Counter of being unable to use a precompiled header
    CantUsePch           = 27,
    /// Counter of being called for pre-processing
    PreProcessing        = 28,
    /// Counter of cache cleanups performed
    NumCleanUps          = 29,
    /// Counter of unsupported code directives
    UnsupportedDirective = 30,
    /// Counter of when the stats were last zeroed
    ZeroTimeStamp        = 31,
}

// 100.0,   1 -> Rolls over from 102399k to 100.0 mb
//  10.0,   2 -> Rolls over from  10239k to  10.00mb
//   5.0,   2 ->                   5119k to   5.00mb
//   1.0,   2 ->                   1023k to   1.00mb
//   0.5,   2 ->                    511k to   0.50mb
const SIZE_SCALE_THRESHOLD: f64 = 10.0;
const SIZE_PRECISION: usize = 2;

impl CacheField {
    /// Return this enums integer as a [usize] for array access purposes.
    pub fn as_usize(self) -> usize { (self as usize) }

    /// Format a given [u64] as per the format type of this field
    ///
    /// ```rust
    /// # use ccache_stats_reader::CacheField;
    /// println!("{}", CacheField::ZeroTimeStamp.format_value(0));
    /// assert_eq!(CacheField::TotalSize.format_value(100), "100 Kb");
    /// ```
    pub fn format_value(self, value: u64) -> String {
        use chrono::{Local, TimeZone};

        match self.metadata().format {
            CacheFieldFormat::None => value.to_string(),
            CacheFieldFormat::TimeStamp => {
                if let Ok(ts) = value.to_string().parse::<i64>() {
                    format!("{}", Local.timestamp(ts, 0))
                } else {
                    format!("{} (ts)", value)
                }
            },
            CacheFieldFormat::SizeTimes1024 => {
                if let Ok(size) = value.to_string().parse::<f64>() {
                    if size < (1024.0 * SIZE_SCALE_THRESHOLD) {
                        format!("{:} Kb", size)
                    } else if size < (1024.0 * 1024.0 * SIZE_SCALE_THRESHOLD)
                    {
                        format!("{:.*} Mb", SIZE_PRECISION, size / 1024.0)
                    } else {
                        format!(
                            "{:.*} Gb",
                            SIZE_PRECISION,
                            size / 1024.0 / 1024.0
                        )
                    }
                } else {
                    format!("{} (kb)", value)
                }
            },
        }
    }

    /// Obtain a [CacheFieldMeta] describing the properties of this field
    pub fn metadata(self) -> &'static CacheFieldMeta {
        match self {
            // Declared in the same order as in stats.c/.cpp
            CacheField::ZeroTimeStamp => &CacheFieldMeta {
                id:      "stats_zeroed_timestamp",
                message: "stats zeroed",
                format:  CacheFieldFormat::TimeStamp,
                flags:   FLAG_ALWAYS,
            },
            CacheField::CacheHitDir => &CacheFieldMeta {
                id:      "direct_cache_hit",
                message: "cache hit (direct)",
                format:  CacheFieldFormat::None,
                flags:   FLAG_ALWAYS,
            },
            CacheField::CacheHitCpp => &CacheFieldMeta {
                id:      "preprocessed_cache_hit",
                message: "cache hit (preprocessed)",
                format:  CacheFieldFormat::None,
                flags:   FLAG_ALWAYS,
            },
            CacheField::ToCache => &CacheFieldMeta {
                id:      "cache_miss",
                message: "cache miss",
                format:  CacheFieldFormat::None,
                flags:   FLAG_ALWAYS,
            },
            CacheField::Link => &CacheFieldMeta {
                id:      "called_for_link",
                message: "called for link",
                format:  CacheFieldFormat::None,
                flags:   FLAG_ALWAYS,
            },
            CacheField::PreProcessing => &CacheFieldMeta {
                id:      "called_for_preprocessing",
                message: "called for preprocessing",
                format:  CacheFieldFormat::None,
                flags:   FLAG_ALWAYS,
            },
            CacheField::Multiple => &CacheFieldMeta {
                id:      "multiple_source_files",
                message: "multiple source files",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::StdOut => &CacheFieldMeta {
                id:      "compiler_produced_stdout",
                message: "compiler produced stdout",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::NoOutput => &CacheFieldMeta {
                id:      "compiler_produced_no_output",
                message: "compiler produced no output",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::EmptyOutput => &CacheFieldMeta {
                id:      "compiler_produced_empty_output",
                message: "compiler produced empty output",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::Status => &CacheFieldMeta {
                id:      "compile_failed",
                message: "compile failed",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::Error => &CacheFieldMeta {
                id:      "internal_error",
                message: "ccache internal error",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::PreProcessor => &CacheFieldMeta {
                id:      "preprocessor_error",
                message: "preprocessor error",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::CantUsePch => &CacheFieldMeta {
                id:      "could_not_use_precompiled_header",
                message: "can't use precompiled header",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::Compiler => &CacheFieldMeta {
                id:      "could_not_find_compiler",
                message: "couldn't find the compiler",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::Missing => &CacheFieldMeta {
                id:      "missing_cache_file",
                message: "cache file missing",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::Args => &CacheFieldMeta {
                id:      "bad_compiler_arguments",
                message: "bad compiler arguments",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::SourceLang => &CacheFieldMeta {
                id:      "unsupported_source_language",
                message: "unsupported source language",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::CompCheck => &CacheFieldMeta {
                id:      "compiler_check_failed",
                message: "compiler check failed",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::ConfTest => &CacheFieldMeta {
                id:      "autoconf_test",
                message: "autoconf compile/link",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::UnsupportedOption => &CacheFieldMeta {
                id:      "unsupported_compiler_option",
                message: "unsupported compiler option",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::UnsupportedDirective => &CacheFieldMeta {
                id:      "unsupported_code_directive",
                message: "unsupported code directive",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::OutStdOut => &CacheFieldMeta {
                id:      "output_to_stdout",
                message: "output to stdout",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::BadOutputFile => &CacheFieldMeta {
                id:      "bad_output_file",
                message: "could not write to output file",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::NoInput => &CacheFieldMeta {
                id:      "no_input_file",
                message: "no input file",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::BadExtraFile => &CacheFieldMeta {
                id:      "error_hashing_extra_file",
                message: "error hashing extra file",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE,
            },
            CacheField::NumCleanUps => &CacheFieldMeta {
                id:      "cleanups_performed",
                message: "cleanups performed",
                format:  CacheFieldFormat::None,
                flags:   FLAG_ALWAYS,
            },
            CacheField::NumFiles => &CacheFieldMeta {
                id:      "files_in_cache",
                message: "files in cache",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NOZERO | FLAG_ALWAYS,
            },
            CacheField::TotalSize => &CacheFieldMeta {
                id:      "cache_size_kibibyte",
                message: "cache size",
                format:  CacheFieldFormat::SizeTimes1024,
                flags:   FLAG_NOZERO | FLAG_ALWAYS,
            },
            CacheField::ObsoleteMaxFiles => &CacheFieldMeta {
                id:      "obsolete_max_files",
                message: "(obsolete) max files",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NOZERO | FLAG_NEVER,
            },
            CacheField::ObsoleteMaxSize => &CacheFieldMeta {
                id:      "obsolete_max_size",
                message: "(obsolete) max size",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NOZERO | FLAG_NEVER,
            },
            CacheField::None => &CacheFieldMeta {
                id:      "internal_none",
                message: "(internal) none",
                format:  CacheFieldFormat::None,
                flags:   FLAG_NONE | FLAG_NEVER,
            },
        }
    }
}

#[cfg_attr(feature = "external_doc", doc(include = "CacheFieldData.md"))]
#[cfg_attr(
    not(feature = "external_doc"),
    doc = "A hash-like interface for accessing values using Enums as keys"
)]
#[derive(Debug, Default, Clone, Copy)]
pub struct CacheFieldData {
    items: [u64; 32],
}

impl CacheFieldData {
    /// Set the stored value for the [CacheField] specified
    ///
    /// ```rust
    /// # use ccache_stats_reader::{CacheField,CacheFieldData};
    /// let mut data: CacheFieldData = Default::default();
    /// data.set_field(CacheField::CacheHitDir, 32);
    /// ```
    pub fn set_field(&mut self, f: CacheField, v: u64) {
        self.items[f.as_usize()] = v
    }

    /// Get a stored value for the [CacheField] specified
    ///
    /// ```rust
    /// # use ccache_stats_reader::{CacheField,CacheFieldData};
    /// # let mut data: CacheFieldData = Default::default();
    /// # data.set_field(CacheField::CacheHitDir, 32);
    /// assert_eq!(data.get_field(CacheField::CacheHitDir), 32);
    /// ```
    pub fn get_field(&self, f: CacheField) -> u64 { self.items[f.as_usize()] }
}

/// Contains an array of [CacheField] in "data order" ( that is, the sequence
/// they should appear in a cache stats file )
pub const FIELD_DATA_ORDER: &[CacheField] = &[
    CacheField::None,
    CacheField::StdOut,
    CacheField::Status,
    CacheField::Error,
    CacheField::ToCache,
    CacheField::PreProcessor,
    CacheField::Compiler,
    CacheField::Missing,
    CacheField::CacheHitCpp,
    CacheField::Args,
    CacheField::Link,
    CacheField::NumFiles,
    CacheField::TotalSize,
    CacheField::ObsoleteMaxFiles,
    CacheField::ObsoleteMaxSize,
    CacheField::SourceLang,
    CacheField::BadOutputFile,
    CacheField::NoInput,
    CacheField::Multiple,
    CacheField::ConfTest,
    CacheField::UnsupportedOption,
    CacheField::OutStdOut,
    CacheField::CacheHitDir,
    CacheField::NoOutput,
    CacheField::EmptyOutput,
    CacheField::BadExtraFile,
    CacheField::CompCheck,
    CacheField::CantUsePch,
    CacheField::PreProcessing,
    CacheField::NumCleanUps,
    CacheField::UnsupportedDirective,
    CacheField::ZeroTimeStamp,
];

/// Contains an array of [CacheField] in "display order" ( that is, the
/// sequence used when pretty-printing for end user consumption ), as defined
/// in `ccache`'s `stats.cpp` (formerly `stats.c` )
pub const FIELD_DISPLAY_ORDER: &[CacheField] = &[
    CacheField::ZeroTimeStamp,
    CacheField::CacheHitDir,
    CacheField::CacheHitCpp,
    CacheField::ToCache,
    CacheField::Link,
    CacheField::PreProcessing,
    CacheField::Multiple,
    CacheField::StdOut,
    CacheField::NoOutput,
    CacheField::EmptyOutput,
    CacheField::Status,
    CacheField::Error,
    CacheField::PreProcessor,
    CacheField::CantUsePch,
    CacheField::Compiler,
    CacheField::Missing,
    CacheField::Args,
    CacheField::SourceLang,
    CacheField::CompCheck,
    CacheField::ConfTest,
    CacheField::UnsupportedOption,
    CacheField::UnsupportedDirective,
    CacheField::OutStdOut,
    CacheField::BadOutputFile,
    CacheField::NoInput,
    CacheField::BadExtraFile,
    CacheField::NumCleanUps,
    CacheField::NumFiles,
    CacheField::TotalSize,
    CacheField::ObsoleteMaxFiles,
    CacheField::ObsoleteMaxSize,
    CacheField::None,
];

#[test]
fn test_cache_field() -> std::io::Result<()> {
    assert_eq!(CacheField::None.as_usize(), 0);
    assert_eq!(CacheField::ZeroTimeStamp.as_usize(), 31);
    Ok(())
}
#[test]
fn test_cache_field_data() -> std::io::Result<()> {
    let mut d: CacheFieldData = Default::default();
    d.set_field(CacheField::None, 1);
    d.set_field(CacheField::ZeroTimeStamp, 2);
    assert_eq!(d.get_field(CacheField::None), 1);
    assert_eq!(d.get_field(CacheField::ZeroTimeStamp), 2);
    Ok(())
}
#[test]
fn test_cache_field_orders() -> std::io::Result<()> {
    match FIELD_DISPLAY_ORDER[0] {
        CacheField::ZeroTimeStamp => {},
        _ => panic!("Display 0 is not ZeroTimeStamp"),
    }
    match FIELD_DATA_ORDER[0] {
        CacheField::None => {},
        _ => panic!("Data 0 is not None"),
    }
    match FIELD_DISPLAY_ORDER[31] {
        CacheField::None => {},
        _ => panic!("Display 31 is not None"),
    }
    match FIELD_DATA_ORDER[31] {
        CacheField::ZeroTimeStamp => {},
        _ => panic!("Data 31 is not ZeroTimeStamp"),
    }
    Ok(())
}

#[test]
fn test_cache_field_metadata() -> std::io::Result<()> {
    assert_eq!(CacheField::None.format_value(0), "0");
    assert_eq!(CacheField::TotalSize.format_value(0), "0 Kb");
    assert_eq!(CacheField::TotalSize.format_value(100), "100 Kb");
    assert_eq!(CacheField::TotalSize.format_value(1_000), "1000 Kb");
    assert_eq!(CacheField::TotalSize.format_value(10_000), "10000 Kb");
    assert_eq!(CacheField::TotalSize.format_value(15_000), "14.65 Mb");
    assert_eq!(CacheField::TotalSize.format_value(150_000), "146.48 Mb");
    assert_eq!(CacheField::TotalSize.format_value(1_500_000), "1464.84 Mb");
    assert_eq!(CacheField::TotalSize.format_value(15_000_000), "14.31 Gb");
    Ok(())
}
