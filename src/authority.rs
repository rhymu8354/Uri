use super::{
    character_classes::{
        REG_NAME_NOT_PCT_ENCODED,
        USER_INFO_NOT_PCT_ENCODED,
    },
    codec::{
        decode_element,
        encode_element,
    },
    context::Context,
    error::Error,
    parse_host_port::parse_host_port,
    validate_ipv6_address::validate_ipv6_address,
};

/// This is the optional part of a URI which governs the URI's namespace.  It
/// typically contains a host name or IP address, and may also include a port
/// number and/or userinfo component.
///
/// # Examples
///
/// ## Parsing an Authority into its components
///
/// ```rust
/// # extern crate rhymuri;
/// use rhymuri::Authority;
///
/// # fn main() -> Result<(), rhymuri::Error> {
/// let authority = Authority::parse("nobody@www.example.com:8080")?;
/// assert_eq!(Some("nobody".as_bytes()), authority.userinfo());
/// assert_eq!("www.example.com".as_bytes(), authority.host());
/// assert_eq!(Some(8080), authority.port());
/// # Ok(())
/// # }
/// ```
///
/// ## Generating a URI from its components
///
/// ```rust
/// # extern crate rhymuri;
/// use rhymuri::Authority;
///
/// # fn main() -> Result<(), rhymuri::Error> {
/// let mut authority = Authority::default();
/// authority.set_userinfo(Some("nobody").map(Into::into));
/// authority.set_host("www.example.com");
/// authority.set_port(Some(8080));
/// assert_eq!("nobody@www.example.com:8080", authority.to_string());
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Authority {
    userinfo: Option<Vec<u8>>,
    host: Vec<u8>,
    port: Option<u16>,
}

impl Authority {
    /// Borrow the host name part of the Authority.
    #[must_use = "why u no use host return value?"]
    pub fn host(&self) -> &[u8] {
        &self.host
    }

    /// Borrow the port number part of the Authority.
    #[must_use = "why did you get the port number and then throw it away?"]
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    /// Change the userinfo part of the Authority.
    pub fn set_userinfo<T>(
        &mut self,
        userinfo: T,
    ) where
        T: Into<Option<Vec<u8>>>,
    {
        self.userinfo = userinfo.into();
    }

    /// Change the host name part of the Authority.
    pub fn set_host<T>(
        &mut self,
        host: T,
    ) where
        T: Into<Vec<u8>>,
    {
        self.host = host.into();
    }

    /// Change the port number part of the Authority.
    pub fn set_port(
        &mut self,
        port: Option<u16>,
    ) {
        self.port = port;
    }

    /// Borrow the userinfo part of the Authority.
    #[must_use = "security breach... security breach... userinfo not used"]
    pub fn userinfo(&self) -> Option<&[u8]> {
        self.userinfo.as_deref()
    }

    /// Interpret the given string as the Authority component of a URI,
    /// separating its various subcomponents, returning an `Authority` value
    /// containing them.
    ///
    /// # Errors
    ///
    /// There are many ways to screw up the Authority part of URI string, and
    /// this function will let you know what's up by returning a variant of the
    /// [`Error`](enum.Error.html) type.
    #[must_use = "you parsed it; don't you want the results?"]
    pub fn parse<T>(authority_string: T) -> Result<Self, Error>
    where
        T: AsRef<str>,
    {
        let (userinfo, host_port_string) =
            Self::parse_userinfo(authority_string.as_ref())?;
        let (host, port) = parse_host_port(host_port_string)?;
        Ok(Self {
            userinfo,
            host,
            port,
        })
    }

    fn parse_userinfo(
        authority: &str
    ) -> Result<(Option<Vec<u8>>, &str), Error> {
        Ok(match authority.find('@') {
            Some(delimiter) => (
                Some(decode_element(
                    &authority[0..delimiter],
                    &USER_INFO_NOT_PCT_ENCODED,
                    Context::Userinfo,
                )?),
                &authority[delimiter + 1..],
            ),
            None => (None, authority),
        })
    }
}

impl std::fmt::Display for Authority {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        if let Some(userinfo) = &self.userinfo {
            write!(
                f,
                "{}@",
                encode_element(&userinfo, &USER_INFO_NOT_PCT_ENCODED)
            )?;
        }
        let host_to_string = String::from_utf8(self.host.clone());
        match host_to_string {
            Ok(host_to_string)
                if validate_ipv6_address(&host_to_string).is_ok() =>
            {
                write!(f, "[{}]", host_to_string.to_ascii_lowercase())?;
            }
            _ => {
                write!(
                    f,
                    "{}",
                    encode_element(&self.host, &REG_NAME_NOT_PCT_ENCODED)
                )?;
            },
        }
        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    // NOTE: These lints are disabled because they're triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::ref_option_ref)]
    #[allow(clippy::from_over_into)]
    fn userinfo() {
        named_tuple!(
            struct TestVector {
                authority_string: &'static str,
                userinfo: Option<&'static str>,
            }
        );
        let test_vectors: &[TestVector] = &[
            ("www.example.com", None).into(),
            ("joe@www.example.com", Some("joe")).into(),
            ("pepe:feelsbadman@www.example.com", Some("pepe:feelsbadman"))
                .into(),
        ];
        for test_vector in test_vectors {
            let authority = Authority::parse(test_vector.authority_string());
            assert!(authority.is_ok());
            let authority = authority.unwrap();
            assert_eq!(
                test_vector.userinfo().map(str::as_bytes),
                authority.userinfo.as_ref().map(|v| &v[..])
            );
        }
    }

    #[test]
    fn userinfo_illegal_characters() {
        let test_vectors = ["%X@www.example.com", "{@www.example.com"];
        for test_vector in &test_vectors {
            let authority = Authority::parse(test_vector);
            assert!(authority.is_err());
        }
    }

    #[test]
    // NOTE: This lint is disabled because it's triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::from_over_into)]
    fn userinfo_barely_legal() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                userinfo: &'static str,
            }
        );
        let test_vectors: &[TestVector] = &[
            ("%41@www.example.com", "A").into(),
            ("@www.example.com", "").into(),
            ("!@www.example.com", "!").into(),
            ("'@www.example.com", "'").into(),
            ("(@www.example.com", "(").into(),
            (";@www.example.com", ";").into(),
            (":@www.example.com", ":").into(),
        ];
        for test_vector in test_vectors {
            let authority = Authority::parse(test_vector.uri_string());
            assert!(authority.is_ok());
            let authority = authority.unwrap();
            assert_eq!(
                Some(test_vector.userinfo().as_bytes()),
                authority.userinfo.as_ref().map(|v| &v[..])
            );
        }
    }

    #[test]
    fn host_illegal_characters() {
        let test_vectors = ["%X@www.example.com", "@www:example.com", "[vX.:]"];
        for test_vector in &test_vectors {
            let authority = Authority::parse(test_vector);
            assert!(authority.is_err());
        }
    }

    #[test]
    // NOTE: This lint is disabled because it's triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::from_over_into)]
    fn host_barely_legal() {
        named_tuple!(
            struct TestVector {
                authority_string: &'static str,
                host: &'static str,
            }
        );
        let test_vectors: &[TestVector] = &[
            ("%41", "a").into(),
            ("", "").into(),
            ("!", "!").into(),
            ("'", "'").into(),
            ("(", "(").into(),
            (";", ";").into(),
            ("1.2.3.4", "1.2.3.4").into(),
            ("[v7.:]", "v7.:").into(),
            ("[v7.aB]", "v7.aB").into(),
        ];
        for test_vector in test_vectors {
            let authority = Authority::parse(test_vector.authority_string());
            assert!(authority.is_ok());
            let authority = authority.unwrap();
            assert_eq!(test_vector.host().as_bytes(), authority.host());
        }
    }

    #[test]
    fn host_ends_in_dot() {
        let authority = Authority::parse("example.com.");
        assert!(authority.is_ok());
        let authority = authority.unwrap();
        assert_eq!(b"example.com.", authority.host());
    }

    #[test]
    fn host_mixed_case() {
        let test_vectors = [
            "www.example.com",
            "www.EXAMPLE.com",
            "www.exAMple.com",
            "www.example.cOM",
            "wWw.exampLe.Com",
        ];
        let normalized_host = "www.example.com";
        for test_vector in &test_vectors {
            let authority = Authority::parse(*test_vector);
            assert!(authority.is_ok());
            let authority = authority.unwrap();
            assert_eq!(normalized_host.as_bytes(), authority.host());
        }
    }
}
