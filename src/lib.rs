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

// This is the character set corresponds to the "query" syntax
// and the "fragment" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
// leaving out "pct-encoded".
lazy_static! {
    static ref QUERY_OR_FRAGMENT_NOT_PCT_ENCODED: HashSet<char> =
        PCHAR_NOT_PCT_ENCODED.iter()
        .chain(['/', '?'].iter())
        .copied()
        .collect::<HashSet<char>>();
}

// This is the character set almost corresponds to the "query" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
// leaving out "pct-encoded", except that '+' is also excluded, because
// for some web services (e.g. AWS S3) a '+' is treated as
// synonymous with a space (' ') and thus gets misinterpreted.
lazy_static! {
    static ref QUERY_NOT_PCT_ENCODED_WITHOUT_PLUS: HashSet<char> =
        UNRESERVED.iter()
        .chain([
            '!', '$', '&', '\'', '(', ')',
            '*', ',', ';', '=',
            ':', '@',
            '/', '?'
        ].iter())
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
    InvalidDecimalOctet,
    TooFewAddressParts,
    TooManyAddressParts,
    TooManyDigits,
    TooManyDoubleColons,
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
            Error::InvalidDecimalOctet => {
                write!(f, "octet group expected")
            },
            Error::TooFewAddressParts => {
                write!(f, "too few address parts")
            },
            Error::TooManyAddressParts => {
                write!(f, "too many address parts")
            },
            Error::TooManyDigits => {
                write!(f, "too many digits")
            },
            Error::TooManyDoubleColons => {
                write!(f, "too many double-colons")
            },
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
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

impl From<std::string::FromUtf8Error> for Error {
    fn from(_: std::string::FromUtf8Error) -> Self {
        Error::IllegalCharacter
    }
}

// TODO: explore possibly returning an iterator instead of a String
fn encode_element(
    element: &[u8],
    allowed_characters: &HashSet<char>
) -> String {
    let mut encoded_element = String::new();
    for ci in element {
        let c = char::try_from(*ci);
        match c {
            Ok(c) if allowed_characters.contains(&c) => {
                encoded_element.push(c);
            },

            _ => {
                encoded_element += &format!("%{:X}", ci);
            }
        }
    }
    encoded_element
}


fn validate_ipv4_address(address: &str) -> Result<(), Error> {
    #[derive(PartialEq)]
    enum State {
        NotInOctet,
        ExpectDigitOrDot,
    }
    let mut num_groups = 0;
    let mut state = State::NotInOctet;
    let mut octet_buffer = String::new();
    // TODO: consider improvements
    //
    // [15:29] silen_z: one cool thing you could consider using (even in
    // the previous function) is matching on tuple of (state, character)
    //
    // Validation of the octet_buffer is done in two places; consider
    // how to remove the redundant code.
    for c in address.chars() {
        state = match state {
            State::NotInOctet if DIGIT.contains(&c) => {
                octet_buffer.push(c);
                State::ExpectDigitOrDot
            },

            State::NotInOctet => {
                return Err(Error::IllegalCharacter);
            },

            State::ExpectDigitOrDot if c == '.' => {
                num_groups += 1;
                // TODO: explore combining these two "if" statements or
                // expressing them in a better way.
                if num_groups > 4 {
                    return Err(Error::TooManyAddressParts);
                }
                if octet_buffer.parse::<u8>().is_err() {
                    return Err(Error::InvalidDecimalOctet);
                }
                octet_buffer.clear();
                State::NotInOctet
            },

            State::ExpectDigitOrDot if DIGIT.contains(&c) => {
                octet_buffer.push(c);
                State::ExpectDigitOrDot
            },

            State::ExpectDigitOrDot => {
                return Err(Error::IllegalCharacter);
            },
        };
    }
    if state == State::NotInOctet {
        return Err(Error::TruncatedHost);
    }
    if !octet_buffer.is_empty() {
        num_groups += 1;
        if octet_buffer.parse::<u8>().is_err() {
            return Err(Error::InvalidDecimalOctet);
        }
    }
    match num_groups {
        4 => Ok(()),
        n if n < 4 => Err(Error::TooFewAddressParts),
        _ => Err(Error::TooManyAddressParts),
    }
}

// TODO: Clippy correctly advises us that this function needs refactoring
// because it has too many lines.  We'll get back to that.
#[allow(clippy::too_many_lines)]
fn validate_ipv6_address(address: &str) -> Result<(), Error> {
    #[derive(PartialEq)]
    enum ValidationState {
        NoGroupsYet,
        ColonButNoGroupsYet,
        AfterDoubleColon,
        InGroupNotIpv4,
        InGroupCouldBeIpv4,
        ColonAfterGroup,
    }
    let mut state = ValidationState::NoGroupsYet;
    let mut num_groups = 0;
    let mut num_digits = 0;
    let mut double_colon_encountered = false;
    let mut potential_ipv4_address_start = 0;
    let mut ipv4_address_encountered = false;
    for (i, c) in address.char_indices() {
        state = match state {
            ValidationState::NoGroupsYet => {
                if c == ':' {
                    ValidationState::ColonButNoGroupsYet
                } else if DIGIT.contains(&c) {
                    potential_ipv4_address_start = i;
                    num_digits = 1;
                    ValidationState::InGroupCouldBeIpv4
                } else if HEXDIG.contains(&c) {
                    num_digits = 1;
                    ValidationState::InGroupNotIpv4
                } else {
                    return Err(Error::IllegalCharacter);
                }
            },

            ValidationState::ColonButNoGroupsYet => {
                if c != ':' {
                    return Err(Error::IllegalCharacter);
                }
                double_colon_encountered = true;
                ValidationState::AfterDoubleColon
            },

            ValidationState::AfterDoubleColon => {
                num_digits += 1;
                if num_digits > 4 {
                    return Err(Error::TooManyDigits);
                }
                if DIGIT.contains(&c) {
                    potential_ipv4_address_start = i;
                    ValidationState::InGroupCouldBeIpv4
                } else if HEXDIG.contains(&c) {
                    ValidationState::InGroupNotIpv4
                } else {
                    return Err(Error::IllegalCharacter);
                }
            },

            ValidationState::InGroupNotIpv4 => {
                if c == ':' {
                    num_digits = 0;
                    num_groups += 1;
                    ValidationState::ColonAfterGroup
                } else if HEXDIG.contains(&c) {
                    num_digits += 1;
                    if num_digits > 4 {
                        return Err(Error::TooManyDigits);
                    }
                    ValidationState::InGroupNotIpv4
                } else {
                    return Err(Error::IllegalCharacter);
                }
            },

            ValidationState::InGroupCouldBeIpv4 => {
                if c == ':' {
                    num_digits = 0;
                    num_groups += 1;
                    ValidationState::ColonAfterGroup
                } else if c == '.' {
                    ipv4_address_encountered = true;
                    break;
                } else {
                    num_digits += 1;
                    if num_digits > 4 {
                        return Err(Error::TooManyDigits);
                    }
                    if DIGIT.contains(&c) {
                        ValidationState::InGroupCouldBeIpv4
                    } else if HEXDIG.contains(&c) {
                        ValidationState::InGroupNotIpv4
                    } else {
                        return Err(Error::IllegalCharacter);
                    }
                }
            },

            ValidationState::ColonAfterGroup => {
                if c == ':' {
                    if double_colon_encountered {
                        return Err(Error::TooManyDoubleColons);
                    } else {
                        double_colon_encountered = true;
                        ValidationState::AfterDoubleColon
                    }
                } else if DIGIT.contains(&c) {
                    potential_ipv4_address_start = i;
                    num_digits += 1;
                    ValidationState::InGroupCouldBeIpv4
                } else if HEXDIG.contains(&c) {
                    num_digits += 1;
                    ValidationState::InGroupNotIpv4
                } else {
                    return Err(Error::IllegalCharacter);
                }
            },
        };
    }
    #[allow(unused_parens)]
    if (
        (state == ValidationState::InGroupNotIpv4)
        || (state == ValidationState::InGroupCouldBeIpv4)
    ) {
        // count trailing group
        num_groups += 1;
    }
    #[allow(unused_parens)]
    if (
        (state == ValidationState::ColonButNoGroupsYet)
        || (state == ValidationState::ColonAfterGroup)
    ) { // trailing single colon
        return Err(Error::TruncatedHost);
    }
    if ipv4_address_encountered {
        validate_ipv4_address(&address[potential_ipv4_address_start..])?;
        num_groups += 2;
    }
    match (double_colon_encountered, num_groups) {
        (true, n) if n <= 7 => Ok(()),
        (false, 8) => Ok(()),
        (_, n) if n > 8 => Err(Error::TooManyAddressParts),
        (_, _) => Err(Error::TooFewAddressParts),
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Authority {
    userinfo: Option<Vec<u8>>,
    host: Vec<u8>,
    port: Option<u16>,
}

impl Authority {
    // TODO: explore possibly making this (and other setters) generic
    // to support *anything* that can be converted implicitly from
    // the type we use to store the information being retrieved.
    #[must_use = "why u no use host return value?"]
    pub fn host(&self) -> &[u8] {
        &self.host
    }

    #[must_use = "why did you get the port number and then throw it away?"]
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn set_userinfo<T>(&mut self, userinfo: Option<T>)
        where Vec<u8>: From<T>
    {
        self.userinfo = userinfo.map(|s| s.into());
    }

    pub fn set_host<T>(&mut self, host: T)
        where Vec<u8>: From<T>
    {
        self.host = host.into();
    }

    pub fn set_port(&mut self, port: Option<u16>) {
        self.port = port;
    }

    #[must_use = "security breach... security breach... userinfo not used"]
    pub fn userinfo(&self) -> Option<&[u8]> {
        self.userinfo.as_deref()
    }
}

impl std::fmt::Display for Authority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(userinfo) = &self.userinfo {
            write!(f, "{}@", encode_element(&userinfo, &USER_INFO_NOT_PCT_ENCODED))?;
        }
        let host_as_string = String::from_utf8(self.host.clone());
        match host_as_string {
            Ok(host_as_string) if validate_ipv6_address(&host_as_string).is_ok() => {
                write!(f, "[{}]", host_as_string.to_ascii_lowercase())?;
            },
            _ => {
                write!(f, "{}", encode_element(&self.host, &REG_NAME_NOT_PCT_ENCODED))?;
            }
        }
        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Uri {
    scheme: Option<String>,
    authority: Option<Authority>,
    path: Vec<Vec<u8>>,
    query: Option<Vec<u8>>,
    fragment: Option<Vec<u8>>,
}

impl Uri {
    #[must_use = "respect mah authoritah"]
    pub fn authority(&self) -> Option<&Authority> {
        self.authority.as_ref()
    }

    fn check_scheme(scheme: &str) -> Result<&str, Error> {
        if scheme.is_empty() {
            return Err(Error::EmptyScheme);
        }
        // TODO: Improve on this by enumerating
        //
        // [16:20] everx80: you could enumerate() and then check the index,
        // instead of having a bool flag?
        let mut is_first_character = true;
        for c in scheme.chars() {
            let valid_characters: &HashSet<char> = if is_first_character {
                &ALPHA
            } else {
                &SCHEME_NOT_FIRST
            };
            if !valid_characters.contains(&c) {
                return Err(Error::IllegalCharacter);
            }
            is_first_character = false;
        }
        Ok(scheme)
    }

    #[must_use = "please use the return value kthxbye"]
    pub fn contains_relative_path(&self) -> bool {
        !Self::is_path_absolute(&self.path)
    }

    fn can_navigate_path_up_one_level(path: &[Vec<u8>]) -> bool {
        match path.first() {
            // First segment empty means path has leading slash,
            // so we can only navigate up if there are two or more segments.
            Some(segment) if segment.is_empty() => path.len() > 1,

            // Otherwise, we can navigate up as long as there is at least one
            // segment.
            Some(_) => true,
            None => false
        }
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
        Self::decode_element(
            query_or_fragment,
            &QUERY_OR_FRAGMENT_NOT_PCT_ENCODED
        )
    }

    #[must_use = "A query and a fragment walked into a bar.  Too bad you're ignoring the fragment because it's actually a funny joke."]
    pub fn fragment(&self) -> Option<&[u8]> {
        self.fragment.as_deref()
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
            Some(authority.host())
        } else {
            None
        }
    }

    fn is_path_absolute(path: &[Vec<u8>]) -> bool {
        match path {
            [segment, ..] if segment.is_empty() => true,
            _ => false
        }
    }

    #[must_use = "why would you call an accessor method and not use the return value, silly human"]
    pub fn is_relative_reference(&self) -> bool {
        self.scheme.is_none()
    }

    pub fn normalize(&mut self) {
        self.path = Self::normalize_path(&self.path);
    }

    // This method applies the "remove_dot_segments" routine talked about
    // in RFC 3986 (https://tools.ietf.org/html/rfc3986) to the path
    // segments of the URI, in order to normalize the path
    // (apply and remove "." and ".." segments).
    fn normalize_path(original_path: &[Vec<u8>]) -> Vec<Vec<u8>> {
        // Rebuild the path one segment
        // at a time, removing and applying special
        // navigation segments ("." and "..") as we go.
        //
        // TODO: The `at_directory_level` variable's purpose
        // is not very clear, and is a bit of a code smell.
        // This probably has something to do with the fact that we
        // represent leading and trailing '/' path separators using
        // empty segments.  Conclusion: We should refactor this.
        let mut at_directory_level = false;
        let mut normalized_path = Vec::new();
        for segment in original_path {
            if segment == b"." {
                at_directory_level = true;
            } else if segment == b".." {
                // Remove last path element
                // if we can navigate up a level.
                if !normalized_path.is_empty() && Self::can_navigate_path_up_one_level(&normalized_path) {
                    normalized_path.pop();
                }
                at_directory_level = true;
            } else {
                // Non-relative elements can just
                // transfer over fine.  An empty
                // segment marks a transition to
                // a directory level context.  If we're
                // already in that context, we
                // want to ignore the transition.
                let new_at_directory_level = segment.is_empty();
                if !at_directory_level || !segment.is_empty() {
                    normalized_path.push(segment.clone());
                }
                at_directory_level = new_at_directory_level;
            }
        }

        // If at the end of rebuilding the path,
        // we're in a directory level context,
        // add an empty segment to mark the fact.
        match (at_directory_level, normalized_path.last()) {
            (true, Some(segment)) if !segment.is_empty() => {
                normalized_path.push(vec![]);
            },
            _ => ()
        }
        normalized_path
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
            Ipv6Address,
            IpvFutureNumber,
            IpvFutureBody,
            GarbageCheck,
            Port,
        };

        // First, check if there is a UserInfo, and if so, extract it.
        let (userinfo, mut host_port_string) = match authority_string.find('@') {
            Some(user_info_delimiter) => (
                Some(
                    Self::decode_element(
                        &authority_string[0..user_info_delimiter],
                        &USER_INFO_NOT_PCT_ENCODED
                    )?
                ),
                &authority_string[user_info_delimiter+1..]
            ),
            None => (
                None,
                authority_string
            )
        };

        // Next, parsing host and port from authority and path.
        let mut port_string = String::new();
        let mut host = Vec::<u8>::new();
        let (mut host_parsing_state, host_is_reg_name) = if host_port_string.starts_with("[v") {
            host_port_string = &host_port_string[2..];
            host.push(b'v');
            (HostParsingState::IpvFutureNumber, false)
        } else if host_port_string.starts_with('[') {
            host_port_string = &host_port_string[1..];
            (HostParsingState::Ipv6Address, false)
        } else {
            (HostParsingState::NotIpLiteral, true)
        };
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

                HostParsingState::Ipv6Address => {
                    if c == ']' {
                        validate_ipv6_address(&ipv6_address)?;
                        host = ipv6_address.chars().map(
                            |c| u8::try_from(c as u32).unwrap()
                        ).collect();
                        HostParsingState::GarbageCheck
                    } else {
                        ipv6_address.push(c);
                        host_parsing_state
                    }
                },

                HostParsingState::IpvFutureNumber => {
                    if c == '.' {
                        host_parsing_state = HostParsingState::IpvFutureBody
                    } else if c == ']' {
                        return Err(Error::TruncatedHost);
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
        path_encoded.into_iter().map(
            |segment| {
                Self::decode_element(&segment, &PCHAR_NOT_PCT_ENCODED)
            }
        )
            .collect::<Result<Vec<Vec<u8>>, Error>>()
    }

    fn parse_query(query_and_or_fragment: &str) -> Result<Option<Vec<u8>>, Error> {
        if query_and_or_fragment.is_empty() {
            Ok(None)
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

    #[must_use = "we went through all that trouble to put the path into a string, and you don't want it?"]
    pub fn path_as_string(&self) -> Result<String, Error> {
        Ok(
            String::from_utf8(
                self.path
                    .join(&b"/"[..])
            )?
        )
    }

    #[must_use = "why did you get the port number and then throw it away?"]
    pub fn port(&self) -> Option<u16> {
        if let Some(authority) = &self.authority {
            authority.port()
        } else {
            None
        }
    }

    #[must_use = "don't you want to know what that query was?"]
    pub fn query(&self) -> Option<&[u8]> {
        self.query.as_deref()
    }

    #[must_use = "why go through all that effort to resolve the URI, when you're not going to use it?!"]
    pub fn resolve(&self, relative_reference: &Self) -> Self {
        // Resolve the reference by following the algorithm
        // from section 5.2.2 in
        // RFC 3986 (https://tools.ietf.org/html/rfc3986).
        let (scheme, authority, path, query) = if relative_reference.scheme.is_some() {
            (
                relative_reference.scheme.clone(),
                relative_reference.authority.clone(),
                Self::normalize_path(&relative_reference.path),
                relative_reference.query.clone()
            )
        } else {
            let scheme = self.scheme.clone();
            if let Some(authority) = &relative_reference.authority {
                (
                    scheme,
                    Some(authority.clone()),
                    Self::normalize_path(&relative_reference.path),
                    relative_reference.query.clone()
                )
            } else {
                let authority = self.authority.clone();
                if relative_reference.path.is_empty() {
                    let path = self.path.clone();
                    let query = if relative_reference.query.is_none() {
                        self.query.clone()
                    } else {
                        relative_reference.query.clone()
                    };
                    (
                        scheme,
                        authority,
                        path,
                        query
                    )
                } else {
                    let query = relative_reference.query.clone();

                    // RFC describes this as:
                    // "if (R.path starts-with "/") then"
                    if Self::is_path_absolute(&relative_reference.path) {
                        (
                            scheme,
                            authority,
                            relative_reference.path.clone(),
                            query
                        )
                    } else {
                        // RFC describes this as:
                        // "T.path = merge(Base.path, R.path);"
                        let mut path = self.path.clone();
                        if path.len() > 1 {
                            path.pop();
                        }
                        path.extend(relative_reference.path.iter().cloned());
                        (
                            scheme,
                            authority,
                            Self::normalize_path(&path),
                            query
                        )
                    }
                }
            }
        };
        Self{
            scheme,
            authority,
            path,
            query,
            fragment: relative_reference.fragment.clone()
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

    pub fn set_authority(&mut self, authority: Option<Authority>) {
        self.authority = authority;
    }

    pub fn set_fragment(&mut self, fragment: Option<&[u8]>) {
        self.fragment = fragment.map(|f| f.into());
    }

    pub fn set_path<'a, T>(&mut self, path: T)
        where T: Iterator<Item=&'a [u8]>
    {
        self.path = path.map(std::borrow::ToOwned::to_owned).collect();
    }

    pub fn set_path_from_str<'a>(&mut self, path: &'a str) {
        if path.is_empty() {
            self.set_path(std::iter::empty());
        } else {
            self.set_path(
                path.split('/').map(str::as_bytes)
            );
        }
    }

    pub fn set_query(&mut self, query: Option<&[u8]>) {
        self.query = query.map(|q| q.into());
    }

    pub fn set_scheme<T>(&mut self, scheme: Option<T>) -> Result<(), Error>
        where String: From<T>
    {
        let scheme: Option<String> = scheme.map(|s| s.into());
        if let Some(scheme) = &scheme {
            Self::check_scheme(scheme)?;
        }
        self.scheme = scheme;
        Ok(())
    }

    fn split_authority_from_path_and_parse_them(
        authority_and_path_string: &str
    ) -> Result<(Option<Authority>, Vec<Vec<u8>>), Error> {
        // Split authority from path.  If there is an authority, parse it.
        if authority_and_path_string.starts_with("//") {
            // Strip off authority marker.
            let authority_and_path_string = &authority_and_path_string[2..];

            // First separate the authority from the path.
            let authority_end = authority_and_path_string.find('/')
                .unwrap_or_else(|| authority_and_path_string.len());
            let authority_string = &authority_and_path_string[0..authority_end];
            let path_string = &authority_and_path_string[authority_end..];

            // Parse the elements inside the authority string.
            let authority = Self::parse_authority(authority_string)?;
            let path = if path_string.is_empty() {
                vec![vec![]]
            } else {
                Self::parse_path(path_string)?
            };
            Ok((Some(authority), path))
        } else {
            let path = Self::parse_path(authority_and_path_string)?;
            Ok((None, path))
        }
    }

    #[must_use = "security breach... security breach... userinfo not used"]
    pub fn userinfo(&self) -> Option<&[u8]> {
        if let Some(authority) = &self.authority {
            authority.userinfo()
        } else {
            None
        }
    }
}

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(scheme) = &self.scheme {
            write!(f, "{}:", scheme)?;
        }
        if let Some(authority) = &self.authority {
            write!(f, "//{}", authority)?;
        }
        // Special case: absolute but otherwise empty path.
        #[allow(unused_parens)]
        if (
            Self::is_path_absolute(&self.path)
            && self.path.len() == 1
        ) {
            write!(f, "/")?;
        }
        for (i, segment) in self.path.iter().enumerate() {
            write!(f, "{}", encode_element(segment, &PCHAR_NOT_PCT_ENCODED))?;
            if i + 1 < self.path.len() {
                write!(f, "/")?;
            }
        }
        if let Some(query) = &self.query {
            write!(f, "?{}", encode_element(query, &QUERY_NOT_PCT_ENCODED_WITHOUT_PLUS))?;
        }
        if let Some(fragment) = &self.fragment {
            write!(f, "#{}", encode_element(fragment, &QUERY_OR_FRAGMENT_NOT_PCT_ENCODED))?;
        }
        Ok(())
    }
}

// TODO: Numerous tests use `Uri::path` when it would be easier to read if they
// used `Uri::path_as_string` instead.

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_from_string_no_scheme() {
        let uri = Uri::parse("foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(None, uri.scheme());
        assert_eq!("foo/bar", uri.path_as_string().unwrap());
        assert_eq!(uri.path_as_string().unwrap(), "foo/bar");
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

    #[test]
    fn parse_from_string_path_corner_cases() {
        struct TestVector<'a> {
            path_in: &'static str,
            path_out: &'a [&'static [u8]],
        };
        let test_vectors = [
            TestVector{path_in: "", path_out: &[]},
            TestVector{path_in: "/", path_out: &[&b""[..]]},
            TestVector{path_in: "/foo", path_out: &[&b""[..], &b"foo"[..]]},
            TestVector{path_in: "foo/", path_out: &[&b"foo"[..], &b""[..]]},
        ];
        for test_vector in &test_vectors {
            let uri = Uri::parse(test_vector.path_in);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(uri.path(), &test_vector.path_out);
        }
    }

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
            let uri = Uri::parse(test_vector.uri_string);
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
        for (test_index, test_vector) in test_vectors.iter().enumerate() {
            let uri = Uri::parse(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                test_vector.contains_relative_path,
                uri.contains_relative_path(),
                "{}", test_index
            );
        }
    }

    #[test]
    fn parse_from_string_query_and_fragment_elements() {
        struct TestVector {
            uri_string: &'static str,
            host: &'static [u8],
            query: Option<&'static [u8]>,
            fragment: Option<&'static [u8]>
        };
        let test_vectors = [
            TestVector{ uri_string: "http://www.example.com/", host: b"www.example.com", query: None, fragment: None },
            TestVector{ uri_string: "http://example.com?foo", host: b"example.com", query: Some(b"foo"), fragment: None },
            TestVector{ uri_string: "http://www.example.com#foo", host: b"www.example.com", query: None, fragment: Some(b"foo") },
            TestVector{ uri_string: "http://www.example.com?foo#bar", host: b"www.example.com", query: Some(b"foo"), fragment: Some(b"bar") },
            TestVector{ uri_string: "http://www.example.com?earth?day#bar", host: b"www.example.com", query: Some(b"earth?day"), fragment: Some(b"bar") },
            TestVector{ uri_string: "http://www.example.com/spam?foo#bar", host: b"www.example.com", query: Some(b"foo"), fragment: Some(b"bar" )},
            TestVector{ uri_string: "http://www.example.com/?", host: b"www.example.com", query: Some(b""), fragment: None },
        ];
        for (test_index, test_vector) in test_vectors.iter().enumerate() {
            let uri = Uri::parse(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(Some(test_vector.host), uri.host());
            assert_eq!(
                test_vector.query,
                uri.query(),
                "{}", test_index
            );
            assert_eq!(test_vector.fragment, uri.fragment());
        }
    }

    #[test]
    fn parse_from_string_user_info() {
        struct TestVector {
            uri_string: &'static str,
            userinfo: Option<&'static [u8]>,
        };
        let test_vectors = [
            TestVector{ uri_string: "http://www.example.com/", userinfo: None },
            TestVector{ uri_string: "http://joe@www.example.com", userinfo: Some(b"joe")},
            TestVector{ uri_string: "http://pepe:feelsbadman@www.example.com", userinfo: Some(b"pepe:feelsbadman") },
            TestVector{ uri_string: "//www.example.com", userinfo: None },
            TestVector{ uri_string: "//bob@www.example.com", userinfo: Some(b"bob") },
            TestVector{ uri_string: "/", userinfo: None },
            TestVector{ uri_string: "foo", userinfo: None },
        ];
        for test_vector in &test_vectors {
            let uri = Uri::parse(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(test_vector.userinfo, uri.userinfo());
        }
    }

    #[test]
    fn parse_from_string_twice_first_user_info_then_without() {
        let uri = Uri::parse("http://joe@www.example.com/foo/bar");
        assert!(uri.is_ok());
        let uri = Uri::parse("/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(None, uri.userinfo());
    }

    #[test]
    fn parse_from_string_scheme_illegal_characters() {
        let test_vectors = [
            "://www.example.com/",
            "0://www.example.com/",
            "+://www.example.com/",
            "@://www.example.com/",
            ".://www.example.com/",
            "h@://www.example.com/",
        ];
        for test_vector in &test_vectors {
            let uri = Uri::parse(*test_vector);
            assert!(uri.is_err());
        }
    }

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
            let uri = Uri::parse(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(Some(test_vector.scheme), uri.scheme());
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
            let uri = Uri::parse(test_vector);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(Some("http"), uri.scheme());
        }
    }

    #[test]
    fn parse_from_string_user_info_illegal_characters() {
        let test_vectors = [
            "//%X@www.example.com/",
            "//{@www.example.com/",
        ];
        for test_vector in &test_vectors {
            let uri = Uri::parse(test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_user_info_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            userinfo: &'static [u8]
        };
        let test_vectors = [
            TestVector{ uri_string: "//%41@www.example.com/", userinfo: b"A" },
            TestVector{ uri_string: "//@www.example.com/", userinfo: b"" },
            TestVector{ uri_string: "//!@www.example.com/", userinfo: b"!" },
            TestVector{ uri_string: "//'@www.example.com/", userinfo: b"'" },
            TestVector{ uri_string: "//(@www.example.com/", userinfo: b"(" },
            TestVector{ uri_string: "//;@www.example.com/", userinfo: b";" },
            TestVector{ uri_string: "http://:@www.example.com/", userinfo: b":" },
        ];
        for test_vector in &test_vectors {
            let uri = Uri::parse(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(Some(test_vector.userinfo), uri.userinfo());
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
            let uri = Uri::parse(test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_host_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            host: &'static [u8]
        };
        let test_vectors = [
            TestVector{ uri_string: "//%41/", host: b"a" },
            TestVector{ uri_string: "///", host: b"" },
            TestVector{ uri_string: "//!/", host: b"!" },
            TestVector{ uri_string: "//'/", host: b"'" },
            TestVector{ uri_string: "//(/", host: b"(" },
            TestVector{ uri_string: "//;/", host: b";" },
            TestVector{ uri_string: "//1.2.3.4/", host: b"1.2.3.4" },
            TestVector{ uri_string: "//[v7.:]/", host: b"v7.:" },
            TestVector{ uri_string: "//[v7.aB]/", host: b"v7.aB" },
        ];
        for test_vector in &test_vectors {
            let uri = Uri::parse(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(Some(test_vector.host), uri.host());
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
        let normalized_host = &b"www.example.com"[..];
        for test_vector in &test_vectors {
            let uri = Uri::parse(*test_vector);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(Some(normalized_host), uri.host());
        }
    }

    #[test]
    fn parse_from_string_host_ends_in_dot() {
        let uri = Uri::parse("http://example.com./foo");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some(&b"example.com."[..]), uri.host());
    }

    #[test]
    fn parse_from_string_dont_misinterpret_colon_in_other_places_as_scheme_delimiter() {
        let test_vectors = [
            "//foo:bar@www.example.com/",
            "//www.example.com/a:b",
            "//www.example.com/foo?a:b",
            "//www.example.com/foo#a:b",
            "//[v7.:]/",
            "/:/foo",
        ];
        for test_vector in &test_vectors {
            let uri = Uri::parse(test_vector);
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
            let uri = Uri::parse(test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_path_barely_legal() {
        struct TestVector<'a> {
            uri_string: &'static str,
            path: &'a [&'static [u8]]
        };
        let test_vectors = [
            TestVector{ uri_string: "/:/foo", path: &[&b""[..], &b":"[..], &b"foo"[..]] },
            TestVector{ uri_string: "bob@/foo", path: &[&b"bob@"[..], &b"foo"[..]] },
            TestVector{ uri_string: "hello!", path: &[&b"hello!"[..]] },
            TestVector{ uri_string: "urn:hello,%20w%6Frld", path: &[&b"hello, world"[..]] },
            TestVector{ uri_string: "//example.com/foo/(bar)/", path: &[&b""[..], &b"foo"[..], &b"(bar)"[..], &b""[..]] },
        ];
        for test_vector in &test_vectors {
            let uri = Uri::parse(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            let path = uri.path().clone();
            assert_eq!(test_vector.path, path);
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
            let uri = Uri::parse(test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_query_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            query: &'static [u8]
        };
        let test_vectors = [
            TestVector{ uri_string: "/?:/foo", query: b":/foo" },
            TestVector{ uri_string: "?bob@/foo", query: b"bob@/foo" },
            TestVector{ uri_string: "?hello!", query: b"hello!" },
            TestVector{ uri_string: "urn:?hello,%20w%6Frld", query: b"hello, world" },
            TestVector{ uri_string: "//example.com/foo?(bar)/", query: b"(bar)/" },
            TestVector{ uri_string: "http://www.example.com/?foo?bar", query: b"foo?bar" },
        ];
        for (test_index, test_vector) in test_vectors.iter().enumerate() {
            let uri = Uri::parse(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(test_vector.query),
                uri.query(),
                "{}", test_index
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
            let uri = Uri::parse(test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_fragment_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            fragment: &'static [u8]
        };
        let test_vectors = [
            TestVector{ uri_string: "/#:/foo", fragment: b":/foo" },
            TestVector{ uri_string: "#bob@/foo", fragment: b"bob@/foo" },
            TestVector{ uri_string: "#hello!", fragment: b"hello!" },
            TestVector{ uri_string: "urn:#hello,%20w%6Frld", fragment: b"hello, world" },
            TestVector{ uri_string: "//example.com/foo#(bar)/", fragment: b"(bar)/" },
            TestVector{ uri_string: "http://www.example.com/#foo?bar", fragment: b"foo?bar" },
        ];
        for test_vector in &test_vectors {
            let uri = Uri::parse(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(Some(test_vector.fragment), uri.fragment());
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
            TestVector{ uri_string: "%bc", path_first_segment: b"\xBC" },
            TestVector{ uri_string: "%Bc", path_first_segment: b"\xBC" },
            TestVector{ uri_string: "%bC", path_first_segment: b"\xBC" },
            TestVector{ uri_string: "%BC", path_first_segment: b"\xBC" },
            TestVector{ uri_string: "%41%42%43", path_first_segment: b"ABC" },
            TestVector{ uri_string: "%41%4A%43%4b", path_first_segment: b"AJCK" },
        ];
        for test_vector in &test_vectors {
            let uri = Uri::parse(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            let segment = uri.path().first().unwrap().clone();
            assert_eq!(segment, test_vector.path_first_segment);
        }
    }

    // TODO: Fix this test
    // #[test]
    // fn normalize_path() {
    //     struct TestVector {
    //         uri_string: &'static str,
    //         normalized_path_segments: &'static [&'static [u8]],
    //     };
    //     let test_vectors = [
    //         TestVector{ uri_string: "/a/b/c/./../../g", normalized_path_segments: &["a", "g"] },
    //         TestVector{ uri_string: "mid/content=5/../6", normalized_path_segments: &["mid", "6"] },
    //         TestVector{ uri_string: "http://example.com/a/../b", normalized_path_segments: &["b"] },
    //         TestVector{ uri_string: "http://example.com/../b", normalized_path_segments: &["b"] },
    //         TestVector{ uri_string: "http://example.com/a/../b/", normalized_path_segments: &["b", ""] },
    //         TestVector{ uri_string: "http://example.com/a/../../b", normalized_path_segments: &["b"] },
    //         TestVector{ uri_string: "./a/b", normalized_path_segments: &["a", "b"] },
    //         TestVector{ uri_string: "..", normalized_path_segments: &[""] },
    //         TestVector{ uri_string: "/", normalized_path_segments: &[""]},
    //         TestVector{ uri_string: "a/b/..", normalized_path_segments: &["a", ""] },
    //         TestVector{ uri_string: "a/b/.", normalized_path_segments: &["a", "b", ""] },
    //         TestVector{ uri_string: "a/b/./c", normalized_path_segments: &["a", "b", "c"] },
    //         TestVector{ uri_string: "a/b/./c/", normalized_path_segments: &["a", "b", "c", ""] },
    //         TestVector{ uri_string: "/a/b/..", normalized_path_segments: &["a", ""]},
    //         TestVector{ uri_string: "/a/b/.", normalized_path_segments: &["a", "b", ""]},
    //         TestVector{ uri_string: "/a/b/./c", normalized_path_segments: &["a", "b", "c"]},
    //         TestVector{ uri_string: "/a/b/./c/", normalized_path_segments: &["a", "b", "c", ""]},
    //         TestVector{ uri_string: "./a/b/..", normalized_path_segments: &["a", ""] },
    //         TestVector{ uri_string: "./a/b/.", normalized_path_segments: &["a", "b", ""] },
    //         TestVector{ uri_string: "./a/b/./c", normalized_path_segments: &["a", "b", "c"] },
    //         TestVector{ uri_string: "./a/b/./c/", normalized_path_segments: &["a", "b", "c", ""] },
    //         TestVector{ uri_string: "../a/b/..", normalized_path_segments: &["a", ""] },
    //         TestVector{ uri_string: "../a/b/.", normalized_path_segments: &["a", "b", ""] },
    //         TestVector{ uri_string: "../a/b/./c", normalized_path_segments: &["a", "b", "c"] },
    //         TestVector{ uri_string: "../a/b/./c/", normalized_path_segments: &["a", "b", "c", ""] },
    //         TestVector{ uri_string: "../a/b/../c", normalized_path_segments: &["a", "c"] },
    //         TestVector{ uri_string: "../a/b/./../c/", normalized_path_segments: &["a", "c", ""] },
    //         TestVector{ uri_string: "../a/b/./../c", normalized_path_segments: &["a", "c"] },
    //         TestVector{ uri_string: "../a/b/./../c/", normalized_path_segments: &["a", "c", ""] },
    //         TestVector{ uri_string: "../a/b/.././c/", normalized_path_segments: &["a", "c", ""] },
    //         TestVector{ uri_string: "../a/b/.././c", normalized_path_segments: &["a", "c"] },
    //         TestVector{ uri_string: "../a/b/.././c/", normalized_path_segments: &["a", "c", ""] },
    //         TestVector{ uri_string: "/./c/d", normalized_path_segments: &["c", "d"]},
    //         TestVector{ uri_string: "/../c/d", normalized_path_segments: &["c", "d"]},
    //     ];
    //     for test_vector in test_vectors.iter() {
    //         let uri = Uri::parse(test_vector.uri_string);
    //         assert!(uri.is_ok());
    //         let uri = uri.unwrap();
    //         uri.normalize_path();
    //         assert_eq!(
    //             test_vector.normalized_path_segments,
    //             uri.path()
    //         );
    //     }
    // }

    #[test]
    fn construct_normalize_and_compare_equivalent_uris() {
        // This was inspired by section 6.2.2
        // of RFC 3986 (https://tools.ietf.org/html/rfc3986).
        let uri1 = Uri::parse("example://a/b/c/%7Bfoo%7D");
        assert!(uri1.is_ok());
        let uri1 = uri1.unwrap();
        let uri2 = Uri::parse("eXAMPLE://a/./b/../b/%63/%7bfoo%7d");
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
            let base_uri = Uri::parse(test_vector.base_string).unwrap();
            let relative_reference_uri = Uri::parse(test_vector.relative_reference_string).unwrap();
            let expected_target_uri = Uri::parse(test_vector.target_string).unwrap();
            let actual_target_uri = base_uri.resolve(&relative_reference_uri);
            assert_eq!(expected_target_uri, actual_target_uri);
        }
    }

    #[test]
    fn empty_path_in_uri_with_authority_is_equivalent_to_slash_only_path() {
        let uri1 = Uri::parse("http://example.com");
        assert!(uri1.is_ok());
        let uri1 = uri1.unwrap();
        let uri2 = Uri::parse("http://example.com/");
        assert!(uri2.is_ok());
        let uri2 = uri2.unwrap();
        assert_eq!(uri1, uri2);
        let uri1 = Uri::parse("//example.com");
        assert!(uri1.is_ok());
        let uri1 = uri1.unwrap();
        let uri2 = Uri::parse("//example.com/");
        assert!(uri2.is_ok());
        let uri2 = uri2.unwrap();
        assert_eq!(uri1, uri2);
    }

    #[test]
    fn ipv6_address_good() {
        struct TestVector {
            uri_string: &'static str,
            expected_host: &'static [u8],
        };
        let test_vectors = [
            TestVector{ uri_string: "http://[::1]/", expected_host: b"::1" },
            TestVector{ uri_string: "http://[::ffff:1.2.3.4]/", expected_host: b"::ffff:1.2.3.4" },
            TestVector{ uri_string: "http://[2001:db8:85a3:8d3:1319:8a2e:370:7348]/", expected_host: b"2001:db8:85a3:8d3:1319:8a2e:370:7348" },
            TestVector{ uri_string: "http://[fFfF::1]", expected_host: b"fFfF::1" },
            TestVector{ uri_string: "http://[1234::1]", expected_host: b"1234::1" },
            TestVector{ uri_string: "http://[fFfF:1:2:3:4:5:6:a]", expected_host: b"fFfF:1:2:3:4:5:6:a" },
            TestVector{ uri_string: "http://[2001:db8:85a3::8a2e:0]/", expected_host: b"2001:db8:85a3::8a2e:0" },
            TestVector{ uri_string: "http://[2001:db8:85a3:8a2e::]/", expected_host: b"2001:db8:85a3:8a2e::" },
        ];
        for test_vector in &test_vectors {
            let uri = Uri::parse(test_vector.uri_string);
            assert!(uri.is_ok());
            assert_eq!(Some(test_vector.expected_host), uri.unwrap().host());
        }
    }

    #[test]
    fn ipv6_address_bad() {
        struct TestVector {
            uri_string: &'static str,
            expected_error: Error,
        };
        let test_vectors = [
            TestVector{ uri_string: "http://[::fFfF::1]", expected_error: Error::TooManyDoubleColons },
            TestVector{ uri_string: "http://[::ffff:1.2.x.4]/", expected_error: Error::IllegalCharacter },
            TestVector{ uri_string: "http://[::ffff:1.2.3.4.8]/", expected_error: Error::TooManyAddressParts },
            TestVector{ uri_string: "http://[::ffff:1.2.3]/", expected_error: Error::TooFewAddressParts },
            TestVector{ uri_string: "http://[::ffff:1.2.3.]/", expected_error: Error::TruncatedHost },
            TestVector{ uri_string: "http://[::ffff:1.2.3.256]/", expected_error: Error::InvalidDecimalOctet },
            TestVector{ uri_string: "http://[::fxff:1.2.3.4]/", expected_error: Error::IllegalCharacter },
            TestVector{ uri_string: "http://[::ffff:1.2.3.-4]/", expected_error: Error::IllegalCharacter },
            TestVector{ uri_string: "http://[::ffff:1.2.3. 4]/", expected_error: Error::IllegalCharacter },
            TestVector{ uri_string: "http://[::ffff:1.2.3.4 ]/", expected_error: Error::IllegalCharacter },
            TestVector{ uri_string: "http://[::ffff:1.2.3.4/", expected_error: Error::TruncatedHost },
            TestVector{ uri_string: "http://::ffff:1.2.3.4]/", expected_error: Error::IllegalCharacter },
            TestVector{ uri_string: "http://::ffff:a.2.3.4]/", expected_error: Error::IllegalCharacter },
            TestVector{ uri_string: "http://::ffff:1.a.3.4]/", expected_error: Error::IllegalCharacter },
            TestVector{ uri_string: "http://[2001:db8:85a3:8d3:1319:8a2e:370:7348:0000]/", expected_error: Error::TooManyAddressParts },
            TestVector{ uri_string: "http://[2001:db8:85a3:8d3:1319:8a2e:370:7348::1]/", expected_error: Error::TooManyAddressParts },
            TestVector{ uri_string: "http://[2001:db8:85a3::8a2e:0:]/", expected_error: Error::TruncatedHost },
            TestVector{ uri_string: "http://[2001:db8:85a3::8a2e::]/", expected_error: Error::TooManyDoubleColons },
            TestVector{ uri_string: "http://[]/", expected_error: Error::TooFewAddressParts },
            TestVector{ uri_string: "http://[:]/", expected_error: Error::TruncatedHost },
            TestVector{ uri_string: "http://[v]/", expected_error: Error::TruncatedHost },
        ];
        for test_vector in &test_vectors {
            let uri = Uri::parse(test_vector.uri_string);
            assert_eq!(
                test_vector.expected_error,
                uri.unwrap_err(),
                "{}",
                test_vector.uri_string
            );
        }
    }

    #[test]
    fn generate_string() {
        struct TestVector {
            scheme: Option<&'static str>,
            userinfo: Option<&'static [u8]>,
            host: Option<&'static [u8]>,
            port: Option<u16>,
            path: &'static str,
            query: Option<&'static [u8]>,
            fragment: Option<&'static [u8]>,
            expected_uri_string: &'static str
        };
        let test_vectors = [
            // general test vectors
            TestVector{ scheme: Some("http"), userinfo: Some(b"bob"), host: Some(b"www.example.com"), port: Some(8080), path: "/abc/def",  query: Some(b"foobar"),   fragment: Some(b"ch2"),   expected_uri_string: "http://bob@www.example.com:8080/abc/def?foobar#ch2" },
            TestVector{ scheme: Some("http"), userinfo: Some(b"bob"), host: Some(b"www.example.com"), port: Some(0), path: "",      query: Some(b"foobar"), fragment: Some(b"ch2"), expected_uri_string: "http://bob@www.example.com:0?foobar#ch2" },
            TestVector{ scheme: Some("http"), userinfo: Some(b"bob"), host: Some(b"www.example.com"), port: Some(0), path: "",      query: Some(b"foobar"), fragment: Some(b""),    expected_uri_string: "http://bob@www.example.com:0?foobar#" },
            TestVector{ scheme: None,         userinfo: None,        host: Some(b"example.com"),     port: None,    path: "",      query: Some(b"bar"),    fragment: None,        expected_uri_string: "//example.com?bar" },
            TestVector{ scheme: None,         userinfo: None,        host: Some(b"example.com"),     port: None,    path: "",      query: Some(b""),       fragment: None,        expected_uri_string: "//example.com?" },
            TestVector{ scheme: None,         userinfo: None,        host: Some(b"example.com"),     port: None,    path: "",      query: None,           fragment: None,        expected_uri_string: "//example.com" },
            TestVector{ scheme: None,         userinfo: None,        host: Some(b"example.com"),     port: None,    path: "/",     query: None,           fragment: None,        expected_uri_string: "//example.com/" },
            TestVector{ scheme: None,         userinfo: None,        host: Some(b"example.com"),     port: None,    path: "/xyz",  query: None,           fragment: None,        expected_uri_string: "//example.com/xyz" },
            TestVector{ scheme: None,         userinfo: None,        host: Some(b"example.com"),     port: None,    path: "/xyz/", query: None,           fragment: None,        expected_uri_string: "//example.com/xyz/" },
            TestVector{ scheme: None,         userinfo: None,        host: None,                    port: None,    path: "/",     query: None,           fragment: None,        expected_uri_string: "/" },
            TestVector{ scheme: None,         userinfo: None,        host: None,                    port: None,    path: "/xyz",  query: None,           fragment: None,        expected_uri_string: "/xyz" },
            TestVector{ scheme: None,         userinfo: None,        host: None,                    port: None,    path: "/xyz/", query: None,           fragment: None,        expected_uri_string: "/xyz/" },
            TestVector{ scheme: None,         userinfo: None,        host: None,                    port: None,    path: "",      query: None,           fragment: None,        expected_uri_string: "" },
            TestVector{ scheme: None,         userinfo: None,        host: None,                    port: None,    path: "xyz",   query: None,           fragment: None,        expected_uri_string: "xyz" },
            TestVector{ scheme: None,         userinfo: None,        host: None,                    port: None,    path: "xyz/",  query: None,           fragment: None,        expected_uri_string: "xyz/" },
            TestVector{ scheme: None,         userinfo: None,        host: None,                    port: None,    path: "",      query: Some(b"bar"),    fragment: None,        expected_uri_string: "?bar" },
            TestVector{ scheme: Some("http"), userinfo: None,        host: None,                    port: None,    path: "",      query: Some(b"bar"),    fragment: None,        expected_uri_string: "http:?bar" },
            TestVector{ scheme: Some("http"), userinfo: None,        host: None,                    port: None,    path: "",      query: None,           fragment: None,        expected_uri_string: "http:" },
            TestVector{ scheme: Some("http"), userinfo: None,        host: Some(b"::1"),           port: None,    path: "",      query: None,           fragment: None,        expected_uri_string: "http://[::1]" },
            TestVector{ scheme: Some("http"), userinfo: None,        host: Some(b"::1.2.3.4"),     port: None,    path: "",      query: None,           fragment: None,        expected_uri_string: "http://[::1.2.3.4]" },
            TestVector{ scheme: Some("http"), userinfo: None,        host: Some(b"1.2.3.4"),         port: None,    path: "",      query: None,           fragment: None,        expected_uri_string: "http://1.2.3.4" },
            TestVector{ scheme: None,         userinfo: None,        host: None,                    port: None,    path: "",      query: None,           fragment: None,        expected_uri_string: "" },
            TestVector{ scheme: Some("http"), userinfo: Some(b"bob"), host: None,                    port: None,    path: "",      query: Some(b"foobar"), fragment: None,        expected_uri_string: "http://bob@?foobar" },
            TestVector{ scheme: None,         userinfo: Some(b"bob"), host: None,                    port: None,    path: "",      query: Some(b"foobar"), fragment: None,        expected_uri_string: "//bob@?foobar" },
            TestVector{ scheme: None,         userinfo: Some(b"bob"), host: None,                    port: None,    path: "",      query: None,           fragment: None,        expected_uri_string: "//bob@" },

            // percent-encoded character test vectors
            TestVector{ scheme: Some("http"), userinfo: Some(b"b b"), host: Some(b"www.example.com"), port: Some(8080), path: "/abc/def", query: Some(b"foobar"),  fragment: Some(b"ch2"), expected_uri_string: "http://b%20b@www.example.com:8080/abc/def?foobar#ch2" },
            TestVector{ scheme: Some("http"), userinfo: Some(b"bob"), host: Some(b"www.e ample.com"), port: Some(8080), path: "/abc/def", query: Some(b"foobar"),  fragment: Some(b"ch2"), expected_uri_string: "http://bob@www.e%20ample.com:8080/abc/def?foobar#ch2" },
            TestVector{ scheme: Some("http"), userinfo: Some(b"bob"), host: Some(b"www.example.com"), port: Some(8080), path: "/a c/def", query: Some(b"foobar"),  fragment: Some(b"ch2"), expected_uri_string: "http://bob@www.example.com:8080/a%20c/def?foobar#ch2" },
            TestVector{ scheme: Some("http"), userinfo: Some(b"bob"), host: Some(b"www.example.com"), port: Some(8080), path: "/abc/def", query: Some(b"foo ar"),  fragment: Some(b"ch2"), expected_uri_string: "http://bob@www.example.com:8080/abc/def?foo%20ar#ch2" },
            TestVector{ scheme: Some("http"), userinfo: Some(b"bob"), host: Some(b"www.example.com"), port: Some(8080), path: "/abc/def", query: Some(b"foobar"),  fragment: Some(b"c 2"), expected_uri_string: "http://bob@www.example.com:8080/abc/def?foobar#c%202" },
            TestVector{ scheme: Some("http"), userinfo: Some(b"bob"), host: Some(b"\xE1\x88\xB4.example.com"),   port: Some(8080), path: "/abc/def",  query: Some(b"foobar"), fragment: None,        expected_uri_string: "http://bob@%E1%88%B4.example.com:8080/abc/def?foobar" },

            // normalization of IPv6 address hex digits
            TestVector{ scheme: Some("http"), userinfo: Some(b"bob"), host: Some(b"fFfF::1"),       port: Some(8080), path: "/abc/def",  query: Some(b"foobar"), fragment: Some(b"c 2"), expected_uri_string: "http://bob@[ffff::1]:8080/abc/def?foobar#c%202" },
        ];
        for test_vector in &test_vectors {
            let mut uri = Uri::default();
            assert!(uri.set_scheme(test_vector.scheme).is_ok());
            #[allow(unused_parens)]
            if (
                test_vector.userinfo.is_some()
                || test_vector.host.is_some()
                || test_vector.port.is_some()
            ) {
                let mut authority = Authority::default();
                authority.set_userinfo(test_vector.userinfo);
                authority.set_host(test_vector.host.unwrap_or_else(|| &b""[..]));
                authority.set_port(test_vector.port);
                uri.set_authority(Some(authority));
            } else {
                uri.set_authority(None);
            }
            uri.set_path_from_str(test_vector.path);
            uri.set_query(test_vector.query);
            uri.set_fragment(test_vector.fragment);
            assert_eq!(
                test_vector.expected_uri_string,
                uri.to_string()
            );
        }
    }

    #[test]
    fn fragment_empty_but_present() {
        let uri = Uri::parse("http://example.com#");
        assert!(uri.is_ok());
        let mut uri = uri.unwrap();
        assert_eq!(Some(&b""[..]), uri.fragment());
        assert_eq!(uri.to_string(), "http://example.com/#");
        uri.set_fragment(None);
        assert_eq!(uri.to_string(), "http://example.com/");
        assert_eq!(None, uri.fragment());

        let uri = Uri::parse("http://example.com");
        assert!(uri.is_ok());
        let mut uri = uri.unwrap();
        assert_eq!(None, uri.fragment());
        uri.set_fragment(Some(&b""[..]));
        assert_eq!(Some(&b""[..]), uri.fragment());
        assert_eq!(uri.to_string(), "http://example.com/#");
    }

    #[test]
    fn query_empty_but_present() {
        let uri = Uri::parse("http://example.com?");
        assert!(uri.is_ok());
        let mut uri = uri.unwrap();
        assert_eq!(Some(&b""[..]), uri.query());
        assert_eq!(uri.to_string(), "http://example.com/?");
        uri.set_query(None);
        assert_eq!(uri.to_string(), "http://example.com/");
        assert_eq!(None, uri.query());

        let uri = Uri::parse("http://example.com");
        assert!(uri.is_ok());
        let mut uri = uri.unwrap();
        assert_eq!(None, uri.query());
        uri.set_query(Some(&b""[..]));
        assert_eq!(Some(&b""[..]), uri.query());
        assert_eq!(uri.to_string(), "http://example.com/?");
    }

    #[test]
    fn make_a_copy() {
        let mut uri1 = Uri::parse("http://www.example.com/foo.txt").unwrap();
        let mut uri2 = uri1.clone();
        uri1.set_query(Some(&b"bar"[..]));
        uri2.set_fragment(Some(&b"page2"[..]));
        let mut uri2_new_auth = uri2.authority().unwrap().clone();
        uri2_new_auth.set_host("example.com");
        uri2.set_authority(Some(uri2_new_auth));
        assert_eq!(uri1.to_string(), "http://www.example.com/foo.txt?bar");
        assert_eq!(uri2.to_string(), "http://example.com/foo.txt#page2");
    }

    #[test]
    fn clear_query() {
        let mut uri = Uri::parse("http://www.example.com/?foo=bar").unwrap();
        uri.set_query(None);
        assert_eq!(uri.to_string(), "http://www.example.com/");
        assert_eq!(None, uri.query());
    }

    #[test]
    fn percent_encode_plus_in_queries() {
        // Although RFC 3986 doesn't say anything about '+', some web services
        // treat it the same as ' ' due to how HTML originally defined how
        // to encode the query portion of a URL
        // (see https://stackoverflow.com/questions/2678551/when-to-encode-space-to-plus-or-20).
        //
        // To avoid issues with these web services, make sure '+' is
        // percent-encoded in a URI when the URI is encoded.
        let mut uri = Uri::default();
        uri.set_query(Some(&b"foo+bar"[..]));
        assert_eq!(uri.to_string(), "?foo%2Bbar");
    }

    #[test]
    fn set_illegal_schemes() {
        let test_vectors = [
            "ab_de",
            "ab/de",
            "ab:de",
            "",
            "&",
            "foo&bar",
        ];
        for test_vector in &test_vectors {
            let mut uri = Uri::default();
            assert!(uri.set_scheme(Some(*test_vector)).is_err());
        }
    }

}
