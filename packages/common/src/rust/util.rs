pub use proc_macro2::{Span, TokenStream};
pub use quote::{format_ident, quote, ToTokens};
pub use syn::ext::IdentExt;
pub use syn::parse::discouraged::Speculative;
pub use syn::parse::{Parse, ParseStream};
pub use syn::spanned::Spanned;
pub use syn::token::{Brace, Bracket, Paren};
pub use syn::visit_mut::VisitMut;
pub use syn::{
    braced, bracketed, parenthesized, Error, Expr, Ident, Lifetime, LitInt, LitStr, Pat,
    Result, Token, Visibility,
};

#[allow(unused_imports)]
pub use crate::{dbg_assert, dbg_quote, parse_quote};

#[cfg(feature = "macro")]
#[allow(unused_imports)]
pub use crate::{diag_warning, diagnostic};

#[macro_export]
macro_rules! parse_quote {
    ( $ty:ty, $($tt:tt)* ) => {
        syn::parse2::<$ty>(quote::quote!{ $($tt)* })
            .map_err(|err| panic!("{err:#?}"))
            .unwrap()
    };
}

#[macro_export]
macro_rules! dbg_quote {
    ( $($tt:tt)* ) => {
        dbg!( $crate::parse_quote!($($tt)*))
    };
}

#[macro_export]
macro_rules! dbg_assert {
    ( $actual:expr, $expected:expr  ) => {
        dbg!(&$actual);
        assert_eq!( $actual, $expected )
    };
}

#[cfg(feature = "macro")]
#[macro_export]
macro_rules! diag_warning {
    ( $span:ident, $( $rest:tt )+ ) => {
        $crate::diagnostic!($span, proc_macro::Level::Warning, $( $rest )*)
    };
}
#[cfg(feature = "macro")]
#[macro_export]
macro_rules! diagnostic {
    ( $span:ident, $level:expr, $msg:expr ) => {
        proc_macro::Diagnostic::spanned(syn::spanned::Spanned::span(&$span).unwrap(), $level, $msg)
    };

    ( $span:ident, $level:expr, $fmt:expr, $( $args:expr ),+ ) => {
        proc_macro::Diagnostic::spanned(syn::spanned::Spanned::span(&$span).unwrap(), $level, format!($fmt, $( $args ),*)).emit()
    };
}
