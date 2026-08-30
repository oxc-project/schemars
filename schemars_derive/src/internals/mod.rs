// Adapted from serde_derive_internals 0.29.1 under MIT OR Apache-2.0.
#![allow(dead_code, unexpected_cfgs, unknown_lints, mismatched_lifetime_syntaxes)]

pub mod ast;
pub mod attr;

mod case;
mod check;
mod ctxt;
mod symbol;

use syn::Type;

pub use self::ctxt::Ctxt;

#[derive(Copy, Clone)]
pub enum Derive {
    Serialize,
    Deserialize,
}

pub fn ungroup(mut ty: &Type) -> &Type {
    while let Type::Group(group) = ty {
        ty = &group.elem;
    }
    ty
}
