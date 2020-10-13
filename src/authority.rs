#![warn(clippy::pedantic)]

use super::character_classes::{
    REG_NAME_NOT_PCT_ENCODED,
    USER_INFO_NOT_PCT_ENCODED,
};
use super::codec::{decode_element, encode_element};
use super::context::Context;
use super::error::Error;
use super::parse_host_port::parse_host_port;
use super::validate_ipv6_address::validate_ipv6_address;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Authority {
    userinfo: Option<Vec<u8>>,
    host: Vec<u8>,
    port: Option<u16>,
}

impl Authority {
    #[must_use = "why u no use host return value?"]
    pub fn host(&self) -> &[u8] {
        &self.host
    }

    #[must_use = "why did you get the port number and then throw it away?"]
    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn set_userinfo<T>(&mut self, userinfo: T)
        where T: Into<Option<Vec<u8>>>
    {
        self.userinfo = userinfo.into();
    }

    pub fn set_host<T>(&mut self, host: T)
        where T: Into<Vec<u8>>
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

    #[must_use = "you parsed it; don't you want the results?"]
    pub fn parse<T>(authority_string: T) -> Result<Self, Error>
        where T: AsRef<str>
    {
        let (userinfo, host_port_string) = Self::parse_userinfo(authority_string.as_ref())?;
        let (host, port) = parse_host_port(host_port_string)?;
        Ok(Self{
            userinfo,
            host,
            port,
        })
    }

    fn parse_userinfo(authority: &str) -> Result<(Option<Vec<u8>>, &str), Error> {
        Ok(match authority.find('@') {
            Some(delimiter) => (
                Some(
                    decode_element(
                        &authority[0..delimiter],
                        &USER_INFO_NOT_PCT_ENCODED,
                        Context::Userinfo
                    )?
                ),
                &authority[delimiter+1..]
            ),
            None => (
                None,
                authority
            )
        })
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
