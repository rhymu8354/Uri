#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Context {
    Fragment,
    Host,
    Ipv4Address,
    Ipv6Address,
    IpvFuture,
    Path,
    Query,
    Scheme,
    Userinfo,
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Context::Fragment => {
                write!(f, "fragment")
            },
            Context::Host => {
                write!(f, "host")
            },
            Context::Ipv4Address => {
                write!(f, "IPv4 address")
            },
            Context::Ipv6Address => {
                write!(f, "IPv6 address")
            },
            Context::IpvFuture => {
                write!(f, "IPvFuture")
            },
            Context::Path => {
                write!(f, "path")
            },
            Context::Query => {
                write!(f, "query")
            },
            Context::Scheme => {
                write!(f, "scheme")
            },
            Context::Userinfo => {
                write!(f, "user info")
            },
        }
    }
}
