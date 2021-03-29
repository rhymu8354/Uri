use std::convert::TryFrom;

use super::{
    character_classes::{
        HEXDIG,
        IPV_FUTURE_LAST_PART,
        REG_NAME_NOT_PCT_ENCODED,
    },
    context::Context,
    error::Error,
    percent_encoded_character_decoder::PercentEncodedCharacterDecoder,
    validate_ipv6_address::validate_ipv6_address,
};

struct Shared {
    host: Vec<u8>,
    host_is_reg_name: bool,
    ipv6_address: String,
    pec_decoder: PercentEncodedCharacterDecoder,
    port_string: String,
}

enum State {
    NotIpLiteral(Shared),
    PercentEncodedCharacter(Shared),
    Ipv6Address(Shared),
    IpvFutureNumber(Shared),
    IpvFutureBody(Shared),
    GarbageCheck(Shared),
    Port(Shared),
}

impl State {
    fn finalize(self) -> Result<(Vec<u8>, Option<u16>), Error> {
        match self {
            Self::PercentEncodedCharacter(_)
            | Self::Ipv6Address(_)
            | Self::IpvFutureNumber(_)
            | Self::IpvFutureBody(_) => {
                // truncated or ended early
                Err(Error::TruncatedHost)
            },
            Self::NotIpLiteral(state)
            | Self::GarbageCheck(state)
            | Self::Port(state) => {
                let mut state = state;
                if state.host_is_reg_name {
                    state.host.make_ascii_lowercase();
                }
                let port = if state.port_string.is_empty() {
                    None
                } else {
                    match state.port_string.parse::<u16>() {
                        Ok(port) => Some(port),
                        Err(error) => {
                            return Err(Error::IllegalPortNumber(error));
                        },
                    }
                };
                Ok((state.host, port))
            },
        }
    }

    fn new(host_port_string: &str) -> (Self, &str) {
        let mut shared = Shared {
            host: Vec::<u8>::new(),
            host_is_reg_name: false,
            ipv6_address: String::new(),
            pec_decoder: PercentEncodedCharacterDecoder::new(),
            port_string: String::new(),
        };
        let mut host_port_string = host_port_string;
        if host_port_string.starts_with("[v") {
            host_port_string = &host_port_string[2..];
            shared.host.push(b'v');
            (Self::IpvFutureNumber(shared), host_port_string)
        } else if host_port_string.starts_with('[') {
            host_port_string = &host_port_string[1..];
            (Self::Ipv6Address(shared), host_port_string)
        } else {
            shared.host_is_reg_name = true;
            (Self::NotIpLiteral(shared), host_port_string)
        }
    }

    fn next(
        self,
        c: char,
    ) -> Result<Self, Error> {
        match self {
            Self::NotIpLiteral(state) => Self::next_not_ip_literal(state, c),
            Self::PercentEncodedCharacter(state) => {
                Self::next_percent_encoded_character(state, c)
            },
            Self::Ipv6Address(state) => Self::next_ipv6_address(state, c),
            Self::IpvFutureNumber(state) => {
                Self::next_ipv_future_number(state, c)
            },
            Self::IpvFutureBody(state) => Self::next_ipv_future_body(state, c),
            Self::GarbageCheck(state) => Self::next_garbage_check(state, c),
            Self::Port(state) => Ok(Self::next_port(state, c)),
        }
    }

    fn next_not_ip_literal(
        state: Shared,
        c: char,
    ) -> Result<Self, Error> {
        let mut state = state;
        if c == '%' {
            Ok(Self::PercentEncodedCharacter(state))
        } else if c == ':' {
            Ok(Self::Port(state))
        } else if REG_NAME_NOT_PCT_ENCODED.contains(&c) {
            state.host.push(u8::try_from(c as u32).unwrap());
            Ok(Self::NotIpLiteral(state))
        } else {
            Err(Error::IllegalCharacter(Context::Host))
        }
    }

    fn next_percent_encoded_character(
        state: Shared,
        c: char,
    ) -> Result<Self, Error> {
        let mut state = state;
        // We can't use `Option::map_or` (or `Option::map_or_else`, for similar
        // reasons) in this case because the closure would take ownership of
        // `state`, preventing it from being used to construct the default
        // value.
        #[allow(clippy::option_if_let_else)]
        if let Some(ci) = state.pec_decoder.next(c)? {
            state.host.push(ci);
            Ok(Self::NotIpLiteral(state))
        } else {
            Ok(Self::PercentEncodedCharacter(state))
        }
    }

    fn next_ipv6_address(
        state: Shared,
        c: char,
    ) -> Result<Self, Error> {
        let mut state = state;
        if c == ']' {
            validate_ipv6_address(&state.ipv6_address)?;
            state.host = state
                .ipv6_address
                .chars()
                .map(|c| u8::try_from(c as u32).unwrap())
                .collect();
            Ok(Self::GarbageCheck(state))
        } else {
            state.ipv6_address.push(c);
            Ok(Self::Ipv6Address(state))
        }
    }

    fn next_ipv_future_number(
        state: Shared,
        c: char,
    ) -> Result<Self, Error> {
        let mut state = state;
        if c == '.' {
            state.host.push(b'.');
            Ok(Self::IpvFutureBody(state))
        } else if c == ']' {
            Err(Error::TruncatedHost)
        } else if HEXDIG.contains(&c) {
            state.host.push(u8::try_from(c as u32).unwrap());
            Ok(Self::IpvFutureNumber(state))
        } else {
            Err(Error::IllegalCharacter(Context::IpvFuture))
        }
    }

    fn next_ipv_future_body(
        state: Shared,
        c: char,
    ) -> Result<Self, Error> {
        let mut state = state;
        if c == ']' {
            Ok(Self::GarbageCheck(state))
        } else if IPV_FUTURE_LAST_PART.contains(&c) {
            state.host.push(u8::try_from(c as u32).unwrap());
            Ok(Self::IpvFutureBody(state))
        } else {
            Err(Error::IllegalCharacter(Context::IpvFuture))
        }
    }

    fn next_garbage_check(
        state: Shared,
        c: char,
    ) -> Result<Self, Error> {
        // illegal to have anything else, unless it's a colon,
        // in which case it's a port delimiter
        if c == ':' {
            Ok(Self::Port(state))
        } else {
            Err(Error::IllegalCharacter(Context::Host))
        }
    }

    fn next_port(
        state: Shared,
        c: char,
    ) -> Self {
        let mut state = state;
        state.port_string.push(c);
        Self::Port(state)
    }
}

pub fn parse_host_port<T>(
    host_port_string: T
) -> Result<(Vec<u8>, Option<u16>), Error>
where
    T: AsRef<str>,
{
    let (machine, host_port_string) = State::new(host_port_string.as_ref());
    host_port_string.chars().try_fold(machine, State::next)?.finalize()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn non_empty_port_number() {
        let result = parse_host_port("www.example.com:8080");
        assert!(result.is_ok());
        let (host, port) = result.unwrap();
        assert_eq!(b"www.example.com", &host[..]);
        assert_eq!(Some(8080), port);
    }

    #[test]
    fn empty_port_number() {
        let result = parse_host_port("www.example.com:");
        assert!(result.is_ok());
        let (host, port) = result.unwrap();
        assert_eq!(b"www.example.com", &host[..]);
        assert_eq!(None, port);
    }

    #[test]
    fn no_port_number() {
        let result = parse_host_port("www.example.com");
        assert!(result.is_ok());
        let (host, port) = result.unwrap();
        assert_eq!(b"www.example.com", &host[..]);
        assert_eq!(None, port);
    }

    #[test]
    fn bad_port_number_purly_alphabetic() {
        let result = parse_host_port("www.example.com:spam");
        assert!(result.is_err());
    }

    #[test]
    fn bad_port_number_starts_numeric_ends_alphabetic() {
        let result = parse_host_port("www.example.com:8080spam");
        assert!(result.is_err());
    }

    #[test]
    fn largest_valid_port_number() {
        let result = parse_host_port("www.example.com:65535");
        assert!(result.is_ok());
        let (_, port) = result.unwrap();
        assert_eq!(Some(65535), port);
    }

    #[test]
    fn bad_port_number_too_big() {
        let result = parse_host_port("www.example.com:65536");
        assert!(matches!(result, Err(Error::IllegalPortNumber(_))));
    }

    #[test]
    fn bad_port_number_negative() {
        let result = parse_host_port("www.example.com:-1234");
        assert!(result.is_err());
    }

    #[test]
    fn truncated_host() {
        let test_vectors = ["[::ffff:1.2.3.4/", "[:]/", "[v]/"];
        for test_vector in &test_vectors {
            assert_eq!(
                Err(Error::TruncatedHost),
                parse_host_port(test_vector),
                "{}",
                test_vector
            );
        }
    }

    #[test]
    fn ipv6_address_with_ipv4_part_missing_bracket() {
        assert!(matches!(
            parse_host_port("::ffff:1.2.3.4]"),
            Err(Error::IllegalPortNumber(_))
        ));
    }
}
