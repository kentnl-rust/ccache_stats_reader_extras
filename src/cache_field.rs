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

#[test]
fn test_cache_field() -> std::io::Result<()> {
    let _ = CacheField::None;
    Ok(())
}
