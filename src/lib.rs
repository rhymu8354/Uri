#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

#[macro_use]
extern crate lazy_static;

mod percent_encoded_character_decoder;
use percent_encoded_character_decoder::PercentEncodedCharacterDecoder;

use std::collections::HashSet;
use std::convert::TryFrom;

// This is the character set containing just the alphabetic characters
// from the ASCII character set.
//
// TODO: improvement
// [16:16] silen_z: btw char::is_ascii_letter or something like that exists
//
// [14:49] silmeth: @rhymu8354 you might want to look at once_cell as a nicer
// macro-less replacement for lazystatic!()
lazy_static! {
    static ref ALPHA: HashSet<char> =
        ('a'..='z')
        .chain('A'..='Z')
        .collect::<HashSet<char>>();
}

// This is the character set containing just numbers.
lazy_static! {
    static ref DIGIT: HashSet<char> =
        ('0'..='9')
        .collect::<HashSet<char>>();
}

// This is the character set containing just the characters allowed
// in a hexadecimal digit.
lazy_static! {
    static ref HEXDIG: HashSet<char> =
        ('0'..='9')
        .chain('A'..='F')
        .chain('a'..='f')
        .collect::<HashSet<char>>();
}

// This is the character set corresponds to the "unreserved" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986).
lazy_static! {
    static ref UNRESERVED: HashSet<char> =
        ALPHA.iter()
        .chain(DIGIT.iter())
        .chain(['-', '.', '_', '~'].iter())
        .copied()
        .collect::<HashSet<char>>();
}

// This is the character set corresponds to the "sub-delims" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986).
lazy_static! {
    static ref SUB_DELIMS: HashSet<char> =
        [
            '!', '$', '&', '\'', '(', ')',
            '*', '+', ',', ';', '='
        ]
        .iter()
        .copied()
        .collect::<HashSet<char>>();
}

// This is the character set corresponds to the second part
// of the "scheme" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986).
lazy_static! {
    static ref SCHEME_NOT_FIRST: HashSet<char> =
        ALPHA.iter()
        .chain(DIGIT.iter())
        .chain(['+', '-', '.'].iter())
        .copied()
        .collect::<HashSet<char>>();
}

// This is the character set corresponds to the "pchar" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
// leaving out "pct-encoded".
lazy_static! {
    static ref PCHAR_NOT_PCT_ENCODED: HashSet<char> =
        UNRESERVED.iter()
        .chain(SUB_DELIMS.iter())
        .chain([':', '@'].iter())
        .copied()
        .collect::<HashSet<char>>();
}

// This is the character set corresponds to the "userinfo" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
// leaving out "pct-encoded".
lazy_static! {
    static ref USER_INFO_NOT_PCT_ENCODED: HashSet<char> =
        UNRESERVED.iter()
        .chain(SUB_DELIMS.iter())
        .chain([':'].iter())
        .copied()
        .collect::<HashSet<char>>();
}

// This is the character set corresponds to the "reg-name" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
// leaving out "pct-encoded".
lazy_static! {
    static ref REG_NAME_NOT_PCT_ENCODED: HashSet<char> =
        UNRESERVED.iter()
        .chain(SUB_DELIMS.iter())
        .copied()
        .collect::<HashSet<char>>();
}

// This is the character set corresponds to the last part of
// the "IPvFuture" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986).
lazy_static! {
    static ref IPV_FUTURE_LAST_PART: HashSet<char> =
        UNRESERVED.iter()
        .chain(SUB_DELIMS.iter())
        .chain([':'].iter())
        .copied()
        .collect::<HashSet<char>>();
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    EmptyScheme,
    IllegalCharacter,
    TruncatedHost,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::EmptyScheme => {
                write!(f, "scheme expected but missing")
            },

            Error::IllegalCharacter => {
                write!(f, "illegal character")
            },

            Error::TruncatedHost => {
                write!(f, "truncated host")
            },
        }
    }
}

impl From<percent_encoded_character_decoder::Error> for Error {
    fn from(error: percent_encoded_character_decoder::Error) -> Self {
        match error {
            percent_encoded_character_decoder::Error::IllegalCharacter => {
                Error::IllegalCharacter
            },
        }
    }
}

pub struct Authority {
    userinfo: Option<Vec<u8>>,
    host: Vec<u8>,
    port: Option<u16>,
}

pub struct Uri {
    scheme: Option<String>,
    authority: Option<Authority>,
    path: Vec<Vec<u8>>,
    query: Option<Vec<u8>>,
    fragment: Option<Vec<u8>>,
}

impl Uri {
    fn check_scheme(scheme: &str) -> Result<&str, Error> {
        if scheme.is_empty() {
            return Err(Error::EmptyScheme);
        }
        // TODO: Improve on this by enumerating
        //
        // [16:20] everx80: you could enumerate() and then check the index,
        // instead of having a bool flag?
        let mut is_FirstCharacter = true;
        for c in scheme.chars() {
            let valid_characters: &HashSet<char> = if is_FirstCharacter {
                &ALPHA
            } else {
                &SCHEME_NOT_FIRST
            };
            if !valid_characters.contains(&c) {
                return Err(Error::IllegalCharacter);
            }
            is_FirstCharacter = false;
        }
        Ok(scheme)
    }

    // TODO: look into making element type more flexible
    fn decode_element(
        element: &str,
        allowed_characters: &'static HashSet<char>
    ) -> Result<Vec<u8>, Error> {
        let mut decoding_pec = false;
        let mut output = Vec::<u8>::new();
        let mut pec_decoder = PercentEncodedCharacterDecoder::new();
        // TODO: revisit this and try to use iterators, once I get better at
        // Rust.
        //
        // [13:50] LeinardoSmith: you could do the find_if and set the
        // condition to when you want to exit
        //
        // [13:52] 715209: i found this: https://stackoverflow.com/a/31507194
        for c in element.chars() {
            if decoding_pec {
                if let Some(c) = pec_decoder.next(c)? {
                    decoding_pec = false;
                    output.push(c);
                }
            } else if c == '%' {
                decoding_pec = true;
            } else if allowed_characters.contains(&c) {
                output.push(c as u8);
            } else {
                return Err(Error::IllegalCharacter);
            }
        }
        Ok(output)
    }

    fn decode_query_or_fragment(query_or_fragment: &str) -> Result<Vec<u8>, Error> {
        unimplemented!()
    }

    #[must_use = "why u no use host return value?"]
    pub fn host(&self) -> Option<&[u8]> {
        // Here is another way to do the same thing, but with some Rust-fu.
        // Credit goes to everx80, ABuffSeagull, and silen_z:
        //
        // self.authority
        //     .as_ref()
        //     .and_then(
        //         |authority| authority.host.as_deref()
        //     )
        //
        // * First `as_ref` gets our authority from `&Option<Authority>` into
        //   `Option<&Authority>` (there is an implicit borrow of
        //   `self.authority` first).
        // * Next, `and_then` basically converts `Option<&Authority>`
        //   into `Option<&[u8]>` by leveraging the closure we provide
        //   to convert `&Authority` into `Option<&[u8]>`.
        // * Finally, our closure uses `as_deref` to turn our `Option<Vec<u8>>`
        //   into an `Option<&[u8]>` since Vec<T> implements DeRef with
        //   `Target=[T]`
        if let Some(authority) = &self.authority {
            Some(&authority.host)
        } else {
            None
        }
    }

    pub fn parse(uri_string: &str) -> Result<Uri, Error> {
        let (scheme, rest) = Self::parse_scheme(uri_string)?;

        let path_end = rest.find(&['?', '#'][..])
            .unwrap_or_else(|| rest.len());
//        let path_end = rest.find(|c| "?#".find(c).is_some())
//            .unwrap_or(rest.len());

        let authority_and_path_string = &rest[0..path_end];
        let query_and_or_fragment = &rest[path_end..];
        let (authority, path) = Self::split_authority_from_path_and_parse_them(authority_and_path_string)?;
        let (fragment, possible_query) = Self::parse_fragment(query_and_or_fragment)?;
        let query = Self::parse_query(possible_query)?;
        Ok(Uri{
            scheme,
            authority,
            path,
            query,
            fragment
        })
    }

    // TODO: Needs refactoring, as Clippy dutifully told us.
    #[allow(clippy::too_many_lines)]
    fn parse_authority(authority_string: &str) -> Result<Authority, Error> {
        // These are the various states for the state machine implemented
        // below to correctly split up and validate the URI substring
        // containing the host and potentially a port number as well.
        #[derive(PartialEq)]
        enum HostParsingState {
            NotIpLiteral,
            PercentEncodedCharacter,
            IpLiteral,
            Ipv6Address,
            IpvFutureNumber,
            IpvFutureBody,
            GarbageCheck,
            Port,
        };

        // First, check if there is a UserInfo, and if so, extract it.
        let (userinfo, host_port_string) = match authority_string.find('@') {
            Some(user_info_delimiter) => (
                Some(
                    Self::decode_element(
                        &authority_string[0..user_info_delimiter],
                        &USER_INFO_NOT_PCT_ENCODED
                    )?
                ),
                &authority_string[user_info_delimiter+1..]
            ),
            _ => (
                None,
                authority_string
            )
        };

        // Next, parsing host and port from authority and path.
        let mut port_string = String::new();
        let (mut host_parsing_state, host_is_reg_name) = if host_port_string.starts_with('[') {
            (HostParsingState::IpLiteral, false)
        } else {
            (HostParsingState::NotIpLiteral, true)
        };
        let mut host = Vec::<u8>::new();
        let mut ipv6_address = String::new();
        let mut pec_decoder = PercentEncodedCharacterDecoder::new();
        for c in host_port_string.chars() {
            host_parsing_state = match host_parsing_state {
                HostParsingState::NotIpLiteral => {
                    if c == '%' {
                        HostParsingState::PercentEncodedCharacter
                    } else if c == ':' {
                        HostParsingState::Port
                    } else if REG_NAME_NOT_PCT_ENCODED.contains(&c) {
                        host.push(u8::try_from(c as u32).unwrap());
                        host_parsing_state
                    } else {
                        return Err(Error::IllegalCharacter);
                    }
                },

                HostParsingState::PercentEncodedCharacter => {
                    if let Some(ci) = pec_decoder.next(c)? {
                        host.push(ci);
                        HostParsingState::NotIpLiteral
                    } else {
                        host_parsing_state
                    }
                },

                HostParsingState::IpLiteral => {
                    if c == 'v' {
                        host.push(b'v');
                        HostParsingState::IpvFutureNumber
                    } else {
                        HostParsingState::Ipv6Address
                    }
                },

                HostParsingState::Ipv6Address => {
                    if c == ']' {
                        host = Self::validate_ipv6_address(&ipv6_address)?;
                        HostParsingState::GarbageCheck
                    } else {
                        ipv6_address.push(c);
                        host_parsing_state
                    }
                },

                HostParsingState::IpvFutureNumber => {
                    if c == '.' {
                        host_parsing_state = HostParsingState::IpvFutureBody
                    } else if !HEXDIG.contains(&c) {
                        return Err(Error::IllegalCharacter);
                    }
                    host.push(u8::try_from(c as u32).unwrap());
                    host_parsing_state
                },

                HostParsingState::IpvFutureBody => {
                    if c == ']' {
                        HostParsingState::GarbageCheck
                    } else if IPV_FUTURE_LAST_PART.contains(&c) {
                        host.push(u8::try_from(c as u32).unwrap());
                        host_parsing_state
                    } else {
                        return Err(Error::IllegalCharacter);
                    }
                },

                HostParsingState::GarbageCheck => {
                    // illegal to have anything else, unless it's a colon,
                    // in which case it's a port delimiter
                    if c == ':' {
                        HostParsingState::Port
                    } else {
                        return Err(Error::IllegalCharacter);
                    }
                },

                HostParsingState::Port => {
                    port_string.push(c);
                    host_parsing_state
                },
            }
        }

        // My normal coding style requires extra parentheses for conditionals
        // having multiple parts broken up into different lines, but rust
        // hates it.  Well, sorry rust, but we're going to do it anyway.
        // FeelsUnusedParensMan
        #[allow(unused_parens)]
        if (
            (host_parsing_state != HostParsingState::NotIpLiteral)
            && (host_parsing_state != HostParsingState::GarbageCheck)
            && (host_parsing_state != HostParsingState::Port)
        ) {
            // truncated or ended early
            return Err(Error::TruncatedHost);
        }
        if host_is_reg_name {
            host.make_ascii_lowercase();
        }
        let port = if port_string.is_empty() {
            None
        } else if let Ok(port) = port_string.parse::<u16>() {
            Some(port)
        } else {
            return Err(Error::IllegalCharacter);
        };
        Ok(Authority{
            userinfo,
            host,
            port,
        })
    }

    fn parse_fragment(query_and_or_fragment: &str) -> Result<(Option<Vec<u8>>, &str), Error> {
        if let Some(fragment_delimiter) = query_and_or_fragment.find('#') {
            let fragment = Self::decode_query_or_fragment(
                &query_and_or_fragment[fragment_delimiter+1..]
            )?;
            Ok((
                Some(fragment),
                &query_and_or_fragment[0..fragment_delimiter]
            ))
        } else {
            Ok((
                None,
                query_and_or_fragment
            ))
        }
    }

    fn parse_path(path_string: &str) -> Result<Vec<Vec<u8>>, Error> {
        // TODO: improvement: make an iterator and only collect at the end.
        let mut path_encoded = Vec::<String>::new();
        match path_string {
            "/" => {
                // Special case of a path that is empty but needs a single
                // empty-string element to indicate that it is absolute.
                path_encoded.push("".to_string());
            },

            "" => {
            },

            mut path_string => {
                // TODO: Try out this improvement:
                // [15:49] silen_z: path_string.split('/').collect()
                loop {
                    if let Some(path_delimiter) = path_string.find('/') {
                        path_encoded.push(
                            path_string[0..path_delimiter].to_string()
                        );
                        path_string = &path_string[path_delimiter+1..];
                    } else {
                        path_encoded.push(path_string.to_string());
                        break;
                    }
                }
            }
        }
        // TODO: improvement
        // [15:57] silen_z: collect into Result<Vec<_>, Error>
        path_encoded.into_iter().map(
            |segment| {
                Self::decode_element(&segment, &PCHAR_NOT_PCT_ENCODED)
            }
        )
            .collect::<Result<Vec<Vec<u8>>, Error>>()
    }

    fn parse_query(query_and_or_fragment: &str) -> Result<Option<Vec<u8>>, Error> {
        if query_and_or_fragment.is_empty() {
            Ok(Some(Vec::new()))
        } else {
            let query = Self::decode_query_or_fragment(&query_and_or_fragment[1..])?;
            Ok(Some(query))
        }
    }

    fn parse_scheme(uri_string: &str) -> Result<(Option<String>, &str), Error> {
        // Limit our search so we don't scan into the authority
        // or path elements, because these may have the colon
        // character as well, which we might misinterpret
        // as the scheme delimiter.
        let authority_or_path_delimiter_start = uri_string.find('/')
            .unwrap_or_else(|| uri_string.len());
        if let Some(scheme_end) = &uri_string[0..authority_or_path_delimiter_start].find(':') {
            let scheme = Self::check_scheme(&uri_string[0..*scheme_end])?
                .to_lowercase();
            Ok((Some(scheme), &uri_string[*scheme_end+1..]))
        } else {
            Ok((None, uri_string))
        }
    }

    #[must_use = "you called path() to get the path, so why you no use?"]
    pub fn path(&self) -> &Vec<Vec<u8>> {
        &self.path
    }

    #[must_use = "why did you get the port number and then throw it away?"]
    pub fn port(&self) -> Option<u16> {
        if let Some(authority) = &self.authority {
            authority.port
        } else {
            None
        }
    }

    #[must_use = "you wanted to use that scheme, right?"]
    pub fn scheme(&self) -> Option<&str> {
        // NOTE: This seemingly magic `as_deref` works because of two
        // things that are going on here:
        // 1) String implements DeRef with `str` as the associated type
        //    `Target`, meaning you can use a String in a context requiring
        //    &str, and String does the conversion work.
        // 2) as_deref works by turning `Option<T>` into `Option<&T::Target>`,
        //    requiring T to implement Deref.  In this case T is String.
        self.scheme.as_deref()
    }

    fn split_authority_from_path_and_parse_them(
        authority_and_path_string: &str
    ) -> Result<(Option<Authority>, Vec<Vec<u8>>), Error> {
        // Split authority from path.  If there is an authority, parse it.
        if &authority_and_path_string[0..2] == "//" {
            // Strip off authority marker.
            let authority_and_path_string = &authority_and_path_string[2..];

            // First separate the authority from the path.
            let authority_end = authority_and_path_string.find('/')
                .unwrap_or_else(|| authority_and_path_string.len());
            let authority_string = &authority_and_path_string[0..authority_end];
            let path_string = &authority_and_path_string[authority_end..];

            // Parse the elements inside the authority string.
            let authority = Self::parse_authority(authority_string)?;
            let path = Self::parse_path(path_string)?;
            Ok((Some(authority), path))
        } else {
            let path = Self::parse_path(authority_and_path_string)?;
            Ok((None, path))
        }
    }

    fn validate_ipv6_address(_address: &str) -> Result<Vec<u8>, Error> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn parse_from_string_no_scheme() {
        let uri = Uri::parse("foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(None, uri.scheme());
        // TODO: needs improvement; I don't like having to spam `to_vec`.
        // [15:49] kimundi2016: &b""[..] may also work
        //   Indeed, we could replace `.to_vec()` with `[..]`.
        //
        // Maybe we just make a convenience method we could use like this:
        // assert_eq!("foo/bar", uri.path_as_str());
        assert_eq!(&[&b"foo"[..], &b"bar"[..]].to_vec(), uri.path());
        assert_eq!(uri.path(), &[&b"foo"[..], &b"bar"[..]].to_vec());
    }

    #[test]
    fn parse_from_string_url() {
        let uri = Uri::parse("http://www.example.com/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some("http"), uri.scheme());
        assert_eq!(Some(&b"www.example.com"[..]), uri.host());
        assert_eq!(uri.path(), &[&b""[..], &b"foo"[..], &b"bar"[..]].to_vec());
    }

    #[test]
    fn parse_from_string_urn_default_path_delimiter() {
        let uri = Uri::parse("urn:book:fantasy:Hobbit");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some("urn"), uri.scheme());
        assert_eq!(None, uri.host());
        assert_eq!(uri.path(), &[&b"book:fantasy:Hobbit"[..]].to_vec());
    }

    // TODO: Fix this test!
    // #[test]
    // fn parse_from_string_path_corner_cases() {
    //     struct TestVector {
    //         path_in: &'static str,
    //         path_out: Vec<Vec<u8>>,
    //     };
    //     let test_vectors = [
    //         TestVector{path_in: "", path_out: &[].to_vec()},
    //         TestVector{path_in: "/", path_out: &[&b"/"[..]].to_vec()},
    //         TestVector{path_in: "/foo", path_out: &[&b""[..], &b"foo"[..]].to_vec()},
    //         TestVector{path_in: "foo/", path_out: &[&b"foo"[..], &b""[..]].to_vec()},
    //     ];
    //     for test_vector in &test_vectors {
    //         let uri = Uri::parse(test_vector.path_in);
    //         assert!(uri.is_ok());
    //         let uri = uri.unwrap();
    //         assert_eq!(uri.path(), &test_vector.path_out);
    //     }
    // }

    #[test]
    fn parse_from_string_has_a_port_number() {
        let uri = Uri::parse("http://www.example.com:8080/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some(&b"www.example.com"[..]), uri.host());
        assert_eq!(Some(8080), uri.port());
    }

    #[test]
    fn parse_from_string_does_not_have_a_port_number() {
        let uri = Uri::parse("http://www.example.com/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some(&b"www.example.com"[..]), uri.host());
        assert_eq!(None, uri.port());
    }

    #[test]
    fn parse_from_string_twice_first_with_port_number_then_without() {
        let uri = Uri::parse("http://www.example.com:8080/foo/bar");
        assert!(uri.is_ok());
        let uri = Uri::parse("http://www.example.com/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(None, uri.port());
    }

    #[test]
    fn parse_from_string_bad_port_number_purly_alphabetic() {
        let uri = Uri::parse("http://www.example.com:spam/foo/bar");
        assert!(uri.is_err());
    }

    #[test]
    fn parse_from_string_bad_port_number_starts_numeric_ends_alphabetic() {
        let uri = Uri::parse("http://www.example.com:8080spam/foo/bar");
        assert!(uri.is_err());
    }

    #[test]
    fn parse_from_string_largest_valid_port_number() {
        let uri = Uri::parse("http://www.example.com:65535/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some(65535), uri.port());
    }

    #[test]
    fn parse_from_string_bad_port_number_too_big() {
        let uri = Uri::parse("http://www.example.com:65536/foo/bar");
        assert!(uri.is_err());
    }

    #[test]
    fn parse_from_string_bad_port_number_negative() {
        let uri = Uri::parse("http://www.example.com:-1234/foo/bar");
        assert!(uri.is_err());
    }

    #[test]
    fn parse_from_string_ends_after_authority() {
        let uri = Uri::parse("http://www.example.com");
        assert!(uri.is_ok());
    }

    #[test]
    fn parse_from_string_relative_vs_non_relative_references() {
        struct TestVector {
            uri_string: &'static str,
            is_relative_reference: bool
        };
        let test_vectors = [
            TestVector{ uri_string: "http://www.example.com/", is_relative_reference: false },
            TestVector{ uri_string: "http://www.example.com", is_relative_reference: false },
            TestVector{ uri_string: "/", is_relative_reference: true },
            TestVector{ uri_string: "foo", is_relative_reference: true },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(test_vector.is_relative_reference, uri.is_relative_reference());
        }
    }

    #[test]
    fn parse_from_string_relative_vs_non_relative_paths() {
        struct TestVector {
            uri_string: &'static str,
            contains_relative_path: bool
        };
        let test_vectors = [
            TestVector{ uri_string: "http://www.example.com/", contains_relative_path: false },
            TestVector{ uri_string: "http://www.example.com", contains_relative_path: false },
            TestVector{ uri_string: "/", contains_relative_path: false },
            TestVector{ uri_string: "foo", contains_relative_path: true },

            /*
             * This is only a valid test vector if we understand
             * correctly that an empty string IS a valid
             * "relative reference" URI with an empty path.
             */
             TestVector{ uri_string: "", contains_relative_path: true },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(test_vector.contains_relative_path, uri.is_relative_path_reference());
        }
    }

    #[test]
    fn parse_from_string_query_and_fragment_elements() {
        struct TestVector {
            uri_string: &'static str,
            host: &'static str,
            query: Option<&'static str>,
            fragment: Option<&'static str>
        };
        let test_vectors = [
            TestVector{ uri_string: "http://www.example.com/", host: "www.example.com", query: None, fragment: None },
            TestVector{ uri_string: "http://example.com?foo", host: "example.com", query: Some("foo"), fragment: None },
            TestVector{ uri_string: "http://www.example.com#foo", host: "www.example.com", query: None, fragment: Some("foo") },
            TestVector{ uri_string: "http://www.example.com?foo#bar", host: "www.example.com", query: Some("foo"), fragment: Some("bar") },
            TestVector{ uri_string: "http://www.example.com?earth?day#bar", host: "www.example.com", query: Some("earth?day"), fragment: Some("bar") },
            TestVector{ uri_string: "http://www.example.com/spam?foo#bar", host: "www.example.com", query: Some("foo"), fragment: Some("bar" )},
            TestVector{ uri_string: "http://www.example.com/?", host: "www.example.com", query: Some(""), fragment: None },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(test_vector.host.to_string()),
                uri.host().map(std::string::ToString::to_string)
            );
            assert_eq!(
                test_vector.query.map(std::string::ToString::to_string),
                uri.query().map(std::string::ToString::to_string)
            );
            assert_eq!(
                test_vector.fragment.map(std::string::ToString::to_string),
                uri.fragment().map(std::string::ToString::to_string)
            );
        }
    }

    #[test]
    fn parse_from_string_user_info() {
        struct TestVector {
            uri_string: &'static str,
            username: Option<&'static str>,
            password: Option<&'static str>
        };
        let test_vectors = [
            TestVector{ uri_string: "http://www.example.com/", username: None, password: None },
            TestVector{ uri_string: "http://joe@www.example.com", username: Some("joe"), password: None},
            TestVector{ uri_string: "http://pepe:feelsbadman@www.example.com", username: Some("pepe"), password: Some("feelsbadman") },
            TestVector{ uri_string: "//www.example.com", username: None, password: None },
            TestVector{ uri_string: "//bob@www.example.com", username: Some("bob"), password: None },
            TestVector{ uri_string: "/", username: None, password: None },
            TestVector{ uri_string: "foo", username: None, password: None },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                test_vector.username.map(std::string::ToString::to_string),
                uri.username().map(std::string::ToString::to_string)
            );
            assert_eq!(
                test_vector.password.map(std::string::ToString::to_string),
                uri.password().map(std::string::ToString::to_string)
            );
        }
    }

    #[test]
    fn parse_from_string_twice_first_user_info_then_without() {
        let uri = uriparse::URIReference::try_from("http://joe@www.example.com/foo/bar");
        assert!(uri.is_ok());
        let uri = uriparse::URIReference::try_from("/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(None, uri.username());
    }

    // FIXME: the following test has to be completely commented out
    // because of this bug:
    // https://github.com/sgodwincs/uriparse-rs/issues/15
    //
    // Basically, uriparse doesn't correctly reject input which matches
    // the `path-noscheme` syntax rule.  It permits colon (":") characters
    // in path segments everywhere, despite what the second paragraph
    // of section 3.3 of RFC 3896 has to say:  "In addition, a URI reference
    // (Section 4.1) may be a relative-path reference, in which case the
    // first path segment cannot contain a colon (":") character."
    //
    // #[test]
    // fn parse_from_string_scheme_illegal_characters() {
    //     let test_vectors = [
    //         "://www.example.com/",
    //         "0://www.example.com/",
    //         "+://www.example.com/",
    //         "@://www.example.com/",
    //         ".://www.example.com/",
    //         "h@://www.example.com/",
    //     ];
    //     for test_vector in &test_vectors {
    //         let uri = uriparse::URIReference::try_from(*test_vector);
    //         assert!(uri.is_err());
    //     }
    // }

    #[test]
    fn parse_from_string_scheme_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            scheme: &'static str
        };
        let test_vectors = [
            TestVector{ uri_string: "h://www.example.com/", scheme: "h" },
            TestVector{ uri_string: "x+://www.example.com/", scheme: "x+" },
            TestVector{ uri_string: "y-://www.example.com/", scheme: "y-" },
            TestVector{ uri_string: "z.://www.example.com/", scheme: "z." },
            TestVector{ uri_string: "aa://www.example.com/", scheme: "aa" },
            TestVector{ uri_string: "a0://www.example.com/", scheme: "a0" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(test_vector.scheme),
                uri.scheme().map(uriparse::Scheme::as_str)
            );
        }
    }

    #[test]
    fn parse_from_string_scheme_mixed_case () {
        let test_vectors = [
            "http://www.example.com/",
            "hTtp://www.example.com/",
            "HTTP://www.example.com/",
            "Http://www.example.com/",
            "HttP://www.example.com/",
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some("http"),
                uri.scheme().map(uriparse::Scheme::as_str)
            );
        }
    }

    #[test]
    fn parse_from_string_user_info_illegal_characters() {
        let test_vectors = [
            "//%X@www.example.com/",
            "//{@www.example.com/",
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_user_info_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            username: &'static str
        };
        let test_vectors = [
            TestVector{ uri_string: "//%41@www.example.com/", username: "A" },
            TestVector{ uri_string: "//@www.example.com/", username: "" },
            TestVector{ uri_string: "//!@www.example.com/", username: "!" },
            TestVector{ uri_string: "//'@www.example.com/", username: "'" },
            TestVector{ uri_string: "//(@www.example.com/", username: "(" },
            TestVector{ uri_string: "//;@www.example.com/", username: ";" },
            TestVector{ uri_string: "http://:@www.example.com/", username: "" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(uriparse::Username::try_from(test_vector.username).unwrap()),
                uri.username().map(std::clone::Clone::clone)
            );
        }
    }

    #[test]
    fn parse_from_string_host_illegal_characters() {
        let test_vectors = [
            "//%X@www.example.com/",
            "//@www:example.com/",
            "//[vX.:]/",
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_host_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            host: &'static str
        };
        let test_vectors = [
            TestVector{ uri_string: "//%41/", host: "a" },
            TestVector{ uri_string: "///", host: "" },
            TestVector{ uri_string: "//!/", host: "!" },
            TestVector{ uri_string: "//'/", host: "'" },
            TestVector{ uri_string: "//(/", host: "(" },
            TestVector{ uri_string: "//;/", host: ";" },
            TestVector{ uri_string: "//1.2.3.4/", host: "1.2.3.4" },

            // FIXME: These two test vectors are commented out because
            // uriparse cannot parse them correctly.  Although they are
            // valid syntax, we get `HostError::AddressMechanismNotSupported`.
            //
            // It would be nice if uriparse would delegate responsibility to
            // handle IPvFuture host syntax, but unfortunately it doesn't.
            //
            // TestVector{ uri_string: "//[v7.:]/", host: "v7.:" },
            // TestVector{ uri_string: "//[v7.aB]/", host: "v7.aB" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(uriparse::Host::try_from(test_vector.host).unwrap()),
                uri.host().map(std::clone::Clone::clone)
            );
        }
    }

    #[test]
    fn parse_from_string_host_mixed_case() {
        let test_vectors = [
            "http://www.example.com/",
            "http://www.EXAMPLE.com/",
            "http://www.exAMple.com/",
            "http://www.example.cOM/",
            "http://wWw.exampLe.Com/",
        ];
        let normalized_host = uriparse::Host::try_from("www.example.com").unwrap();
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(&normalized_host),
                uri.host()
            );
        }
    }

    #[test]
    fn parse_from_string_host_ends_in_dot() {
        let uri = uriparse::URIReference::try_from("http://example.com./foo");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(
            Some(&uriparse::Host::try_from("example.com.").unwrap()),
            uri.host()
        );
    }

    #[test]
    fn parse_from_string_dont_misinterpret_colon_in_other_places_as_scheme_delimiter() {
        let test_vectors = [
            "//foo:bar@www.example.com/",
            "//www.example.com/a:b",
            "//www.example.com/foo?a:b",
            "//www.example.com/foo#a:b",

            // FIXME: This test vector is commented out because
            // uriparse cannot parse it correctly.  Although it is
            // valid syntax, we get `HostError::AddressMechanismNotSupported`.
            //
            // It would be nice if uriparse would delegate responsibility to
            // handle IPvFuture host syntax, but unfortunately it doesn't.
            //
            // "//[v7.:]/",

            "/:/foo",
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(None, uri.scheme());
        }
    }

    #[test]
    fn parse_from_string_path_illegal_characters() {
        let test_vectors = [
            "http://www.example.com/foo[bar",
            "http://www.example.com/]bar",
            "http://www.example.com/foo]",
            "http://www.example.com/[",
            "http://www.example.com/abc/foo]",
            "http://www.example.com/abc/[",
            "http://www.example.com/foo]/abc",
            "http://www.example.com/[/abc",
            "http://www.example.com/foo]/",
            "http://www.example.com/[/",
            "/foo[bar",
            "/]bar",
            "/foo]",
            "/[",
            "/abc/foo]",
            "/abc/[",
            "/foo]/abc",
            "/[/abc",
            "/foo]/",
            "/[/",
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_path_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            path: &'static str
        };
        let test_vectors = [
            TestVector{ uri_string: "/:/foo", path: "/:/foo" },
            TestVector{ uri_string: "bob@/foo", path: "bob@/foo" },
            TestVector{ uri_string: "hello!", path: "hello!" },

            // NOTE: uriparse does not do percent encoding for us,
            // and SP (space) is not acceptable in a path, even
            // as input to the `try_from` function.
            //
            // FIXME: For this test vector to pass, we have to normalize
            // the path *after* parsing it from the `uri_string`, despite
            // what the `uriparse` documentation says about percent
            // encoding playing no role in equality checking.
            //
            // https://github.com/sgodwincs/uriparse-rs/issues/16
            TestVector{ uri_string: "urn:hello,%20w%6Frld", path: "hello,%20world" },

            TestVector{ uri_string: "//example.com/foo/(bar)/", path: "/foo/(bar)/" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            let mut path = uri.path().clone();
            path.normalize(false);
            assert_eq!(
                uriparse::Path::try_from(test_vector.path).unwrap(),
                path
            );
        }
    }

    #[test]
    fn parse_from_string_query_illegal_characters() {
        let test_vectors = [
            "http://www.example.com/?foo[bar",
            "http://www.example.com/?]bar",
            "http://www.example.com/?foo]",
            "http://www.example.com/?[",
            "http://www.example.com/?abc/foo]",
            "http://www.example.com/?abc/[",
            "http://www.example.com/?foo]/abc",
            "http://www.example.com/?[/abc",
            "http://www.example.com/?foo]/",
            "http://www.example.com/?[/",
            "?foo[bar",
            "?]bar",
            "?foo]",
            "?[",
            "?abc/foo]",
            "?abc/[",
            "?foo]/abc",
            "?[/abc",
            "?foo]/",
            "?[/",
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_query_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            query: &'static str
        };
        let test_vectors = [
            TestVector{ uri_string: "/?:/foo", query: ":/foo" },
            TestVector{ uri_string: "?bob@/foo", query: "bob@/foo" },
            TestVector{ uri_string: "?hello!", query: "hello!" },

            // NOTE: uriparse does not do percent encoding for us,
            // and SP (space) is not acceptable in a query, even
            // as input to the `try_from` function.
            //
            TestVector{ uri_string: "urn:?hello,%20w%6Frld", query: "hello,%20world" },

            TestVector{ uri_string: "//example.com/foo?(bar)/", query: "(bar)/" },
            TestVector{ uri_string: "http://www.example.com/?foo?bar", query: "foo?bar" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(&uriparse::Query::try_from(test_vector.query).unwrap()),
                uri.query()
            );
        }
    }

    #[test]
    fn parse_from_string_fragment_illegal_characters() {
        let test_vectors = [
            "http://www.example.com/#foo[bar",
            "http://www.example.com/#]bar",
            "http://www.example.com/#foo]",
            "http://www.example.com/#[",
            "http://www.example.com/#abc/foo]",
            "http://www.example.com/#abc/[",
            "http://www.example.com/#foo]/abc",
            "http://www.example.com/#[/abc",
            "http://www.example.com/#foo]/",
            "http://www.example.com/#[/",
            "#foo[bar",
            "#]bar",
            "#foo]",
            "#[",
            "#abc/foo]",
            "#abc/[",
            "#foo]/abc",
            "#[/abc",
            "#foo]/",
            "#[/",
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_fragment_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            fragment: &'static str
        };
        let test_vectors = [
            TestVector{ uri_string: "/#:/foo", fragment: ":/foo" },
            TestVector{ uri_string: "#bob@/foo", fragment: "bob@/foo" },
            TestVector{ uri_string: "#hello!", fragment: "hello!" },

            // NOTE: uriparse does not do percent encoding for us,
            // and SP (space) is not acceptable in a fragment, even
            // as input to the `try_from` function.
            //
            TestVector{ uri_string: "urn:#hello,%20w%6Frld", fragment: "hello,%20world" },

            TestVector{ uri_string: "//example.com/foo#(bar)/", fragment: "(bar)/" },
            TestVector{ uri_string: "http://www.example.com/#foo?bar", fragment: "foo?bar" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(&uriparse::Fragment::try_from(test_vector.fragment).unwrap()),
                uri.fragment()
            );
        }
    }

    #[test]
    fn parse_from_string_paths_with_percent_encoded_characters() {
        struct TestVector {
            uri_string: &'static str,
            path_first_segment: &'static [u8]
        };
        let test_vectors = [
            TestVector{ uri_string: "%41", path_first_segment: b"A" },
            TestVector{ uri_string: "%4A", path_first_segment: b"J" },
            TestVector{ uri_string: "%4a", path_first_segment: b"J" },

            // Note: uriparse refuses to decode the percent encodings
            // of non-ASCII characters, even if they represent valid
            // UTF-8 encodings.  So we have to keep them percent-encoded,
            // unfortunately.
            TestVector{ uri_string: "%bc", path_first_segment: b"%BC" },
            TestVector{ uri_string: "%Bc", path_first_segment: b"%BC" },
            TestVector{ uri_string: "%bC", path_first_segment: b"%BC" },
            TestVector{ uri_string: "%BC", path_first_segment: b"%BC" },

            TestVector{ uri_string: "%41%42%43", path_first_segment: b"ABC" },
            TestVector{ uri_string: "%41%4A%43%4b", path_first_segment: b"AJCK" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            let mut segment = uri.path().segments().first().unwrap().clone();
            segment.normalize();
            assert_eq!(
                segment,
                test_vector.path_first_segment
            );
        }
    }

    #[test]
    fn normalize_path() {
        struct TestVector {
            uri_string: &'static str,
            normalized_path_segments: &'static [&'static str],
            is_absolute: bool
        };
        let test_vectors = [
            TestVector{ uri_string: "/a/b/c/./../../g", normalized_path_segments: &["a", "g"], is_absolute: true },
            TestVector{ uri_string: "mid/content=5/../6", normalized_path_segments: &["mid", "6"], is_absolute: false },
            TestVector{ uri_string: "http://example.com/a/../b", normalized_path_segments: &["b"], is_absolute: true },
            TestVector{ uri_string: "http://example.com/../b", normalized_path_segments: &["b"], is_absolute: true },
            TestVector{ uri_string: "http://example.com/a/../b/", normalized_path_segments: &["b", ""], is_absolute: true },
            TestVector{ uri_string: "http://example.com/a/../../b", normalized_path_segments: &["b"], is_absolute: true },
            TestVector{ uri_string: "./a/b", normalized_path_segments: &["a", "b"], is_absolute: false },
            TestVector{ uri_string: "..", normalized_path_segments: &[""], is_absolute: false },
            TestVector{ uri_string: "/", normalized_path_segments: &[""], is_absolute: true },
            TestVector{ uri_string: "a/b/..", normalized_path_segments: &["a", ""], is_absolute: false },
            TestVector{ uri_string: "a/b/.", normalized_path_segments: &["a", "b", ""], is_absolute: false },
            TestVector{ uri_string: "a/b/./c", normalized_path_segments: &["a", "b", "c"], is_absolute: false },
            TestVector{ uri_string: "a/b/./c/", normalized_path_segments: &["a", "b", "c", ""], is_absolute: false },
            TestVector{ uri_string: "/a/b/..", normalized_path_segments: &["a", ""], is_absolute: true },
            TestVector{ uri_string: "/a/b/.", normalized_path_segments: &["a", "b", ""], is_absolute: true },
            TestVector{ uri_string: "/a/b/./c", normalized_path_segments: &["a", "b", "c"], is_absolute: true },
            TestVector{ uri_string: "/a/b/./c/", normalized_path_segments: &["a", "b", "c", ""], is_absolute: true },
            TestVector{ uri_string: "./a/b/..", normalized_path_segments: &["a", ""], is_absolute: false },
            TestVector{ uri_string: "./a/b/.", normalized_path_segments: &["a", "b", ""], is_absolute: false },
            TestVector{ uri_string: "./a/b/./c", normalized_path_segments: &["a", "b", "c"], is_absolute: false },
            TestVector{ uri_string: "./a/b/./c/", normalized_path_segments: &["a", "b", "c", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/..", normalized_path_segments: &["a", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/.", normalized_path_segments: &["a", "b", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/./c", normalized_path_segments: &["a", "b", "c"], is_absolute: false },
            TestVector{ uri_string: "../a/b/./c/", normalized_path_segments: &["a", "b", "c", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/../c", normalized_path_segments: &["a", "c"], is_absolute: false },
            TestVector{ uri_string: "../a/b/./../c/", normalized_path_segments: &["a", "c", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/./../c", normalized_path_segments: &["a", "c"], is_absolute: false },
            TestVector{ uri_string: "../a/b/./../c/", normalized_path_segments: &["a", "c", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/.././c/", normalized_path_segments: &["a", "c", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/.././c", normalized_path_segments: &["a", "c"], is_absolute: false },
            TestVector{ uri_string: "../a/b/.././c/", normalized_path_segments: &["a", "c", ""], is_absolute: false },
            TestVector{ uri_string: "/./c/d", normalized_path_segments: &["c", "d"], is_absolute: true },
            TestVector{ uri_string: "/../c/d", normalized_path_segments: &["c", "d"], is_absolute: true },
        ];
        for test_vector in test_vectors.iter() {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            let mut path = uri.path().clone();
            path.normalize(false);
            assert_eq!(
                path.segments(),
                test_vector.normalized_path_segments.iter().map(
                    |segment| uriparse::Segment::try_from(*segment).unwrap()
                ).collect::<Vec<uriparse::Segment>>()
            );
            assert_eq!(test_vector.is_absolute, path.is_absolute());
        }
    }

    #[test]
    fn construct_normalize_and_compare_equivalent_uris() {
        // This was inspired by section 6.2.2
        // of RFC 3986 (https://tools.ietf.org/html/rfc3986).
        let uri1 = uriparse::URIReference::try_from("example://a/b/c/%7Bfoo%7D");
        assert!(uri1.is_ok());
        let uri1 = uri1.unwrap();
        let uri2 = uriparse::URIReference::try_from("eXAMPLE://a/./b/../b/%63/%7bfoo%7d");
        assert!(uri2.is_ok());
        let mut uri2 = uri2.unwrap();
        assert_ne!(uri1, uri2);
        uri2.normalize();
        assert_eq!(uri1, uri2);
    }

    #[test]
    fn reference_resolution() {
        struct TestVector {
            base_string: &'static str,
            relative_reference_string: &'static str,
            target_string: &'static str
        };
        let test_vectors = [
            // These are all taken from section 5.4.1
            // of RFC 3986 (https://tools.ietf.org/html/rfc3986).
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g:h", target_string: "g:h" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g", target_string: "http://a/b/c/g" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "./g", target_string: "http://a/b/c/g" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g/", target_string: "http://a/b/c/g/" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "//g", target_string: "http://g" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "?y", target_string: "http://a/b/c/d;p?y" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g?y", target_string: "http://a/b/c/g?y" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "#s", target_string: "http://a/b/c/d;p?q#s" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g#s", target_string: "http://a/b/c/g#s" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g?y#s", target_string: "http://a/b/c/g?y#s" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: ";x", target_string: "http://a/b/c/;x" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g;x", target_string: "http://a/b/c/g;x" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g;x?y#s", target_string: "http://a/b/c/g;x?y#s" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "", target_string: "http://a/b/c/d;p?q" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: ".", target_string: "http://a/b/c/" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "./", target_string: "http://a/b/c/" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "..", target_string: "http://a/b/" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "../", target_string: "http://a/b/" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "../g", target_string: "http://a/b/g" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "../..", target_string: "http://a" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "../../", target_string: "http://a" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "../../g", target_string: "http://a/g" },

            // Here are some examples of our own.
            TestVector{ base_string: "http://example.com", relative_reference_string: "foo", target_string: "http://example.com/foo" },
            TestVector{ base_string: "http://example.com/", relative_reference_string: "foo", target_string: "http://example.com/foo" },
            TestVector{ base_string: "http://example.com", relative_reference_string: "foo/", target_string: "http://example.com/foo/" },
            TestVector{ base_string: "http://example.com/", relative_reference_string: "foo/", target_string: "http://example.com/foo/" },
            TestVector{ base_string: "http://example.com", relative_reference_string: "/foo", target_string: "http://example.com/foo" },
            TestVector{ base_string: "http://example.com/", relative_reference_string: "/foo", target_string: "http://example.com/foo" },
            TestVector{ base_string: "http://example.com", relative_reference_string: "/foo/", target_string: "http://example.com/foo/" },
            TestVector{ base_string: "http://example.com/", relative_reference_string: "/foo/", target_string: "http://example.com/foo/" },
            TestVector{ base_string: "http://example.com/", relative_reference_string: "?foo", target_string: "http://example.com/?foo" },
            TestVector{ base_string: "http://example.com/", relative_reference_string: "#foo", target_string: "http://example.com/#foo" },
        ];
        for test_vector in &test_vectors {
            let base_uri = uriparse::URI::try_from(test_vector.base_string).unwrap();
            let relative_reference_uri = uriparse::URIReference::try_from(test_vector.relative_reference_string).unwrap();
            let expected_target_uri = uriparse::URI::try_from(test_vector.target_string).unwrap();
            let actual_target_uri = base_uri.resolve(&relative_reference_uri);
            assert_eq!(expected_target_uri, actual_target_uri);
        }
    }

    #[test]
    fn empty_path_in_uri_with_authority_is_equivalent_to_slash_only_path() {
        let uri1 = uriparse::URIReference::try_from("http://example.com");
        assert!(uri1.is_ok());
        let uri1 = uri1.unwrap();
        let uri2 = uriparse::URIReference::try_from("http://example.com/");
        assert!(uri2.is_ok());
        let uri2 = uri2.unwrap();
        assert_eq!(uri1, uri2);
        let uri1 = uriparse::URIReference::try_from("//example.com");
        assert!(uri1.is_ok());
        let uri1 = uri1.unwrap();
        let uri2 = uriparse::URIReference::try_from("//example.com/");
        assert!(uri2.is_ok());
        let uri2 = uri2.unwrap();
        assert_eq!(uri1, uri2);
    }

    #[test]
    fn ipv6_address() {
        struct TestVector {
            uri_string: &'static str,
            expected_host: Option<&'static str>
        };
        let test_vectors = [
            // valid
            TestVector{ uri_string: "http://[::1]/", expected_host: Some("[::1]") },

            // FIXME: RFC 3986 allows the least-significant 32-bits of an
            // IPv6 address to be represented in IPv4 address textual
            // format, but unfortunately uriparse doesn't support it.
            //
            // https://github.com/sgodwincs/uriparse-rs/issues/17
            //
            //TestVector{ uri_string: "http://[::ffff:1.2.3.4]/", expected_host: Some("[::ffff:1.2.3.4]") },

            TestVector{ uri_string: "http://[2001:db8:85a3:8d3:1319:8a2e:370:7348]/", expected_host: Some("[2001:db8:85a3:8d3:1319:8a2e:370:7348]") },
            TestVector{ uri_string: "http://[fFfF::1]", expected_host: Some("[fFfF::1]") },
            TestVector{ uri_string: "http://[fFfF:1:2:3:4:5:6:a]", expected_host: Some("[fFfF:1:2:3:4:5:6:a]") },

            // invalid
            TestVector{ uri_string: "http://[::fFfF::1]", expected_host: None },
            TestVector{ uri_string: "http://[::ffff:1.2.x.4]/", expected_host: None },
            TestVector{ uri_string: "http://[::ffff:1.2.3.4.8]/", expected_host: None },
            TestVector{ uri_string: "http://[::ffff:1.2.3]/", expected_host: None },
            TestVector{ uri_string: "http://[::ffff:1.2.3.]/", expected_host: None },
            TestVector{ uri_string: "http://[::ffff:1.2.3.256]/", expected_host: None },
            TestVector{ uri_string: "http://[::fxff:1.2.3.4]/", expected_host: None },
            TestVector{ uri_string: "http://[::ffff:1.2.3.-4]/", expected_host: None },
            TestVector{ uri_string: "http://[::ffff:1.2.3. 4]/", expected_host: None },
            TestVector{ uri_string: "http://[::ffff:1.2.3.4 ]/", expected_host: None },
            TestVector{ uri_string: "http://[::ffff:1.2.3.4/", expected_host: None },
            TestVector{ uri_string: "http://::ffff:1.2.3.4]/", expected_host: None },
            TestVector{ uri_string: "http://::ffff:a.2.3.4]/", expected_host: None },
            TestVector{ uri_string: "http://::ffff:1.a.3.4]/", expected_host: None },
            TestVector{ uri_string: "http://[2001:db8:85a3:8d3:1319:8a2e:370:7348:0000]/", expected_host: None },
            TestVector{ uri_string: "http://[2001:db8:85a3::8a2e:0:]/", expected_host: None },
            TestVector{ uri_string: "http://[2001:db8:85a3::8a2e::]/", expected_host: None },
            TestVector{ uri_string: "http://[]/", expected_host: None },
            TestVector{ uri_string: "http://[:]/", expected_host: None },
            TestVector{ uri_string: "http://[v]/", expected_host: None },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            if let Some(host) = test_vector.expected_host {
                assert!(uri.is_ok());
                assert_eq!(
                    &uriparse::Host::try_from(host).unwrap(),
                    uri.unwrap().host().unwrap()
                );
            } else {
                assert!(uri.is_err());
            }
        }
    }

    #[test]
    fn generate_string() {
        struct TestVector {
            scheme: Option<&'static str>,
            username: Option<&'static str>,
            host: Option<&'static str>,
            port: Option<u16>,
            path: &'static str,
            query: Option<&'static str>,
            fragment: Option<&'static str>,
            expected_uri_string: &'static str
        };
        let test_vectors = [
            // general test vectors
            TestVector{ scheme: Some("http"), username: Some("bob"), host: Some("www.example.com"), port: Some(8080), path: "/abc/def",  query: Some("foobar"),   fragment: Some("ch2"),   expected_uri_string: "http://bob@www.example.com:8080/abc/def?foobar#ch2" },

            // NOTE: uriparse unnecessarily adds a '/' character to the path for these cases.
            // Technically it's not an error, but it differs from our C++ implementation
            // and adds an unnecessary character.
            TestVector{ scheme: Some("http"), username: Some("bob"), host: Some("www.example.com"), port: Some(0), path: "",      query: Some("foobar"), fragment: Some("ch2"), expected_uri_string: "http://bob@www.example.com:0/?foobar#ch2" },
            TestVector{ scheme: Some("http"), username: Some("bob"), host: Some("www.example.com"), port: Some(0), path: "",      query: Some("foobar"), fragment: Some(""),    expected_uri_string: "http://bob@www.example.com:0/?foobar#" },
            TestVector{ scheme: None,         username: None,        host: Some("example.com"),     port: None,    path: "",      query: Some("bar"),    fragment: None,        expected_uri_string: "//example.com/?bar" },
            TestVector{ scheme: None,         username: None,        host: Some("example.com"),     port: None,    path: "",      query: Some(""),       fragment: None,        expected_uri_string: "//example.com/?" },
            TestVector{ scheme: None,         username: None,        host: Some("example.com"),     port: None,    path: "",      query: None,           fragment: None,        expected_uri_string: "//example.com/" },

            TestVector{ scheme: None,         username: None,        host: Some("example.com"),     port: None,    path: "/",     query: None,           fragment: None,        expected_uri_string: "//example.com/" },
            TestVector{ scheme: None,         username: None,        host: Some("example.com"),     port: None,    path: "/xyz",  query: None,           fragment: None,        expected_uri_string: "//example.com/xyz" },
            TestVector{ scheme: None,         username: None,        host: Some("example.com"),     port: None,    path: "/xyz/", query: None,           fragment: None,        expected_uri_string: "//example.com/xyz/" },
            TestVector{ scheme: None,         username: None,        host: None,                    port: None,    path: "/",     query: None,           fragment: None,        expected_uri_string: "/" },
            TestVector{ scheme: None,         username: None,        host: None,                    port: None,    path: "/xyz",  query: None,           fragment: None,        expected_uri_string: "/xyz" },
            TestVector{ scheme: None,         username: None,        host: None,                    port: None,    path: "/xyz/", query: None,           fragment: None,        expected_uri_string: "/xyz/" },
            TestVector{ scheme: None,         username: None,        host: None,                    port: None,    path: "",      query: None,           fragment: None,        expected_uri_string: "" },
            TestVector{ scheme: None,         username: None,        host: None,                    port: None,    path: "xyz",   query: None,           fragment: None,        expected_uri_string: "xyz" },
            TestVector{ scheme: None,         username: None,        host: None,                    port: None,    path: "xyz/",  query: None,           fragment: None,        expected_uri_string: "xyz/" },
            TestVector{ scheme: None,         username: None,        host: None,                    port: None,    path: "",      query: Some("bar"),    fragment: None,        expected_uri_string: "?bar" },
            TestVector{ scheme: Some("http"), username: None,        host: None,                    port: None,    path: "",      query: Some("bar"),    fragment: None,        expected_uri_string: "http:?bar" },
            TestVector{ scheme: Some("http"), username: None,        host: None,                    port: None,    path: "",      query: None,           fragment: None,        expected_uri_string: "http:" },

            // NOTE: uriparse unnecessarily adds a '/' character to the path for this case.
            // Technically it's not an error, but it differs from our C++ implementation
            // and adds an unnecessary character.
            TestVector{ scheme: Some("http"), username: None, host: Some("[::1]"), port: None, path: "", query: None, fragment: None, expected_uri_string: "http://[::1]/" },

            // FIXME: RFC 3986 allows the least-significant 32-bits of an
            // IPv6 address to be represented in IPv4 address textual
            // format, but unfortunately uriparse doesn't support it.
            //
            // https://github.com/sgodwincs/uriparse-rs/issues/17
            //
            // TestVector{ scheme: Some("http"), username: None,        host: Some("[::1.2.3.4]"),     port: None,       path: "",          query: None,             fragment: None,          expected_uri_string: "http://[::1.2.3.4]/" },

            TestVector{ scheme: Some("http"), username: None, host: Some("1.2.3.4"), port: None, path: "", query: None, fragment: None, expected_uri_string: "http://1.2.3.4/" },
            TestVector{ scheme: None,         username: None, host: None,            port: None, path: "", query: None, fragment: None, expected_uri_string: "" },

            // Note: Because uriparse requires a host to emit any authority,
            // we have to use some empty string for host to signal that
            // we want to include an authority when we are building the URI.
            //
            // NOTE: uriparse unnecessarily adds a '/' character to the path for this case.
            // Technically it's not an error, but it differs from our C++ implementation
            // and adds an unnecessary character.
            TestVector{ scheme: Some("http"), username: Some("bob"), host: Some(""), port: None, path: "", query: Some("foobar"), fragment: None, expected_uri_string: "http://bob@/?foobar" },
            TestVector{ scheme: None,         username: Some("bob"), host: Some(""), port: None, path: "", query: Some("foobar"), fragment: None, expected_uri_string: "//bob@/?foobar" },
            TestVector{ scheme: None,         username: Some("bob"), host: Some(""), port: None, path: "", query: None,           fragment: None, expected_uri_string: "//bob@/" },

            // percent-encoded character test vectors
            //
            // NOTE: uriparse does not do the percent-encoding for us,
            // but we can still verify that the URI builder works with them.
            TestVector{ scheme: Some("http"), username: Some("b%20b"), host: Some("www.example.com"),   port: Some(8080), path: "/abc/def",   query: Some("foobar"),   fragment: Some("ch2"),   expected_uri_string: "http://b%20b@www.example.com:8080/abc/def?foobar#ch2" },
            TestVector{ scheme: Some("http"), username: Some("bob"),   host: Some("www.e%20ample.com"), port: Some(8080), path: "/abc/def",   query: Some("foobar"),   fragment: Some("ch2"),   expected_uri_string: "http://bob@www.e%20ample.com:8080/abc/def?foobar#ch2" },
            TestVector{ scheme: Some("http"), username: Some("bob"),   host: Some("www.example.com"),   port: Some(8080), path: "/a%20c/def", query: Some("foobar"),   fragment: Some("ch2"),   expected_uri_string: "http://bob@www.example.com:8080/a%20c/def?foobar#ch2" },
            TestVector{ scheme: Some("http"), username: Some("bob"),   host: Some("www.example.com"),   port: Some(8080), path: "/abc/def",   query: Some("foo%20ar"), fragment: Some("ch2"),   expected_uri_string: "http://bob@www.example.com:8080/abc/def?foo%20ar#ch2" },
            TestVector{ scheme: Some("http"), username: Some("bob"),   host: Some("www.example.com"),   port: Some(8080), path: "/abc/def",   query: Some("foobar"),   fragment: Some("c%202"), expected_uri_string: "http://bob@www.example.com:8080/abc/def?foobar#c%202" },

            // Note: uriparse refuses to decode the percent encodings
            // of non-ASCII characters, even if they represent valid
            // UTF-8 encodings.  So we have to keep them percent-encoded,
            // unfortunately.
            TestVector{ scheme: Some("http"), username: Some("bob"), host: Some("%E1%88%B4.example.com"),   port: Some(8080), path: "/abc/def",  query: Some("foobar"),   fragment: None,          expected_uri_string: "http://bob@%E1%88%B4.example.com:8080/abc/def?foobar" },

            // normalization of IPv6 address hex digits
            TestVector{ scheme: Some("http"), username: Some("bob"), host: Some("[fFfF::1]"),       port: Some(8080), path: "/abc/def",  query: Some("foobar"),   fragment: Some("c%202"), expected_uri_string: "http://bob@[ffff::1]:8080/abc/def?foobar#c%202" },
        ];
        for test_vector in &test_vectors {
            let mut uri_builder = uriparse::URIReferenceBuilder::new();
            uri_builder
                .scheme(test_vector.scheme.map(|scheme| uriparse::Scheme::try_from(scheme).unwrap()))
                .authority(
                    match test_vector.host {
                        None => None,
                        Some(host) => Some(
                            uriparse::Authority::from_parts(
                                test_vector.username.map(|username| uriparse::Username::try_from(username).unwrap()),
                                None::<uriparse::Password>,
                                uriparse::Host::try_from(host).unwrap(),
                                test_vector.port
                            ).unwrap()
                        )
                    }
                )
                .path(uriparse::Path::try_from(test_vector.path).unwrap())
                .query(test_vector.query.map(|query| uriparse::Query::try_from(query).unwrap()))
                .fragment(test_vector.fragment.map(|fragment| uriparse::Fragment::try_from(fragment).unwrap()));
            let uri = uri_builder.build().unwrap();
            assert_eq!(
                test_vector.expected_uri_string,
                uri.to_string()
            );
        }
    }

    #[test]
    fn fragment_empty_but_present() {
        let uri = uriparse::URIReference::try_from("http://example.com#");
        assert!(uri.is_ok());
        let mut uri = uri.unwrap();
        assert_eq!(
            Some(&uriparse::Fragment::try_from("").unwrap()),
            uri.fragment()
        );
        assert_eq!(uri.to_string(), "http://example.com/#");
        uri.set_fragment(None::<uriparse::Fragment>).unwrap();
        assert_eq!(uri.to_string(), "http://example.com/");
        assert_eq!(None, uri.fragment());

        let uri = uriparse::URIReference::try_from("http://example.com");
        assert!(uri.is_ok());
        let mut uri = uri.unwrap();
        assert_eq!(None, uri.fragment());
        uri.set_fragment(Some("")).unwrap();
        assert_eq!(
            Some(&uriparse::Fragment::try_from("").unwrap()),
            uri.fragment()
        );
        assert_eq!(uri.to_string(), "http://example.com/#");
    }

    #[test]
    fn query_empty_but_present() {
        let uri = uriparse::URIReference::try_from("http://example.com?");
        assert!(uri.is_ok());
        let mut uri = uri.unwrap();
        assert_eq!(
            Some(&uriparse::Query::try_from("").unwrap()),
            uri.query()
        );
        assert_eq!(uri.to_string(), "http://example.com/?");
        uri.set_query(None::<uriparse::Query>).unwrap();
        assert_eq!(uri.to_string(), "http://example.com/");
        assert_eq!(None, uri.query());

        let uri = uriparse::URIReference::try_from("http://example.com");
        assert!(uri.is_ok());
        let mut uri = uri.unwrap();
        assert_eq!(None, uri.query());
        uri.set_query(Some("")).unwrap();
        assert_eq!(
            Some(&uriparse::Query::try_from("").unwrap()),
            uri.query()
        );
        assert_eq!(uri.to_string(), "http://example.com/?");
    }

    #[test]
    fn make_a_copy() {
        let mut uri1 = uriparse::URIReference::try_from("http://www.example.com/foo.txt").unwrap();
        let mut uri2 = uri1.clone();
        uri1.set_query(Some("bar")).unwrap();
        uri2.set_fragment(Some("page2")).unwrap();
        let mut uri2_new_auth = uri2.authority().unwrap().clone();
        uri2_new_auth.set_host("example.com").unwrap();
        uri2.set_authority(Some(uri2_new_auth)).unwrap();
        assert_eq!(uri1.to_string(), "http://www.example.com/foo.txt?bar");
        assert_eq!(uri2.to_string(), "http://example.com/foo.txt#page2");
    }

    #[test]
    fn clear_query() {
        let mut uri = uriparse::URIReference::try_from("http://www.example.com/?foo=bar").unwrap();
        uri.set_query(None::<uriparse::Query>).unwrap();
        assert_eq!(uri.to_string(), "http://www.example.com/");
        assert_eq!(None, uri.query());
    }

    // NOTE: The following test is commented out because uriparse does not
    // percent-encode '+' for us.
    //
    // #[test]
    // fn percent_encode_plus_in_queries() {
    //     // Although RFC 3986 doesn't say anything about '+', some web services
    //     // treat it the same as ' ' due to how HTML originally defined how
    //     // to encode the query portion of a URL
    //     // (see https://stackoverflow.com/questions/2678551/when-to-encode-space-to-plus-or-20).
    //     //
    //     // To avoid issues with these web services, make sure '+' is
    //     // percent-encoded in a URI when the URI is encoded.
    //     let mut uri = uriparse::URIReference::try_from("").unwrap();
    //     uri.set_query(Some("foo+bar")).unwrap();
    //     assert_eq!(uri.to_string(), "?foo%2Bbar");
    // }

}
