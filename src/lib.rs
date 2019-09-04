#![cfg_attr(feature = "external_doc", feature(external_doc))]
#![cfg_attr(feature = "non_exhaustive", feature(non_exhaustive))]
#![cfg_attr(feature = "external_doc", doc(include = "lib.md"))]
#![cfg_attr(
    not(feature = "external_doc"),
    doc = "This crate implements a simple interface for accessing `ccache` \
           stats without needing an `exec` call."
)]

mod cache_field;
pub use cache_field::{CacheField, CacheFieldData};
