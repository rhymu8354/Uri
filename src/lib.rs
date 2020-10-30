//! This crate implements [IETF RFC 3986](https://tools.ietf.org/html/rfc3986),
//! "Uniform Resource Identifier (URI): Generic Syntax".  The [`Uri`] type
//! can be used to parse and generate RFC-conformant URI strings to and from
//! their various components.
//!
//! A Uniform Resource Identifier (URI) is a compact sequence of characters
//! that identifies an abstract or physical resource.  One common form of URI
//! is the Uniform Resource Locator (URL), used to reference web resources:
//!
//! ```text
//! http://www.example.com/foo?bar#baz
//! ```
//!
//! Another kind of URI is the path reference:
//!
//! ```text
//! /usr/bin/zip
//! ```
//!
//! # Examples
//!
//! ## Parsing a URI into its components
//!
//! ```rust
//! # extern crate rhymuri;
//! use rhymuri::Uri;
//!
//! let uri = Uri::parse("http://www.example.com/foo?bar#baz").unwrap();
//! let authority = uri.authority().unwrap();
//! assert_eq!("www.example.com".as_bytes(), authority.host());
//! assert_eq!(
//!     Some("www.example.com"),
//!     uri.host_to_string().unwrap().as_deref()
//! );
//! assert_eq!("/foo", uri.path_to_string().unwrap());
//! assert_eq!(Some("bar"), uri.query_to_string().unwrap().as_deref());
//! assert_eq!(Some("baz"), uri.fragment_to_string().unwrap().as_deref());
//! ```
//!
//! ## Generating a URI from its components
//!
//! ```rust
//! # extern crate rhymuri;
//! use rhymuri::{
//!     Authority,
//!     Uri,
//! };
//!
//! let mut uri = Uri::default();
//! assert!(uri.set_scheme(String::from("http")).is_ok());
//! let mut authority = Authority::default();
//! authority.set_host("www.example.com");
//! uri.set_authority(Some(authority));
//! uri.set_path_from_str("/foo");
//! uri.set_query(Some("bar".into()));
//! uri.set_fragment(Some("baz".into()));
//! assert_eq!("http://www.example.com/foo?bar#baz", uri.to_string());
//! ```
//!
//! [`Uri`]: struct.Uri.html

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
mod uri;
mod validate_ipv4_address;
mod validate_ipv6_address;

pub use crate::{
    authority::Authority,
    context::Context,
    error::Error,
    uri::Uri,
};
