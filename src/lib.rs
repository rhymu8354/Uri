#![warn(clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]
#![warn(missing_docs)]

#[cfg(test)]
#[macro_use]
extern crate named_tuple;

mod authority;
mod character_classes;
mod codec;
mod context;
mod error;
mod parse_host_port;
mod percent_encoded_character_decoder;
mod validate_ipv4_address;
mod validate_ipv6_address;
mod uri;

pub use crate::authority::Authority;
pub use crate::uri::Uri;
