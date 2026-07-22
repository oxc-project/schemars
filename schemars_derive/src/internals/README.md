This directory contains the subset of `serde_derive_internals` 0.29.1 used by
`oxc_schemars_derive`.

It is vendored because the published crate depends on Syn 2, which makes its
syntax-tree types incompatible with Syn 3. The Syn 3 adjustments match
[serde-rs/serde#3085](https://github.com/serde-rs/serde/pull/3085). The unused
receiver-rewriting modules are omitted.

The vendored code is available under either the MIT or Apache-2.0 license; see
`LICENSE-MIT` and `LICENSE-APACHE` in this directory.
