A hash-like interface for accessing values using Enums as keys.

This struct is basically a cosmetic wrapper that
provides a bit of compile time safety, improved performance
over [HashMap](std::collections::HashMap) access, improved
performance over [HashMap](std::collections::HashMap)
memory utilization, while still having most of the same
conveniences.

```rust

use ccache_stats_reader::{CacheField, CacheFieldData};
let mut data : CacheFieldData = Default::default();
data.set_field(CacheField::CacheHitDir, 32);
assert_eq!(data.get_field(CacheField::CacheHitCpp), 0);

```

