use super::context::Context;

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("URI contains non-UTF8 sequences")]
    CannotExpressAsUtf8(#[from] std::string::FromUtf8Error),

    #[error("scheme expected but missing")]
    EmptyScheme,

    #[error("illegal character in {0}")]
    IllegalCharacter(Context),

    #[error("illegal percent encoding")]
    IllegalPercentEncoding,

    #[error("illegal port number")]
    IllegalPortNumber(#[source] std::num::ParseIntError),

    #[error("octet group expected")]
    InvalidDecimalOctet,

    #[error("too few address parts")]
    TooFewAddressParts,

    #[error("too many address parts")]
    TooManyAddressParts,

    #[error("too many digits in IPv6 address part")]
    TooManyDigits,

    #[error("too many double-colons in IPv6 address")]
    TooManyDoubleColons,

    #[error("truncated host")]
    TruncatedHost,
}

