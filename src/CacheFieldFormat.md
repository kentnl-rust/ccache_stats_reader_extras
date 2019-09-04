A descriptor enum for which formatting to use for a given field.

All [CacheField]'s have an associated [CacheFieldFormat]
that decides how to convert the field value (a [u64]) into
presentation text.

For most, this just passes them through as-is, but some are
defined to be "byte size" fields ( which can be pretty
printed in a compact form like 1.0mb ), or "unix time"
fields (which can be pretty printed in local-time)
