/// This enumerates the various places where an error might occur parsing a
/// URI.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Context {
    /// This is the fragment of the URI, such as `#baz` in
    /// `http://www.example.com/foo?bar#baz`.
    Fragment,

    /// This is the host name of the URI, such as `www.example.com` in
    /// `http://www.example.com/foo?bar#baz`.
    Host,

    /// This is the IPv4 portion of the IPv6 host name in the URI, such as
    /// `1.2.3.4` in `http://[::ffff:1.2.3.4]/foo?bar#baz`.
    Ipv4Address,

    /// This is the IPv6 host name in the URI, such as
    /// `::ffff:1.2.3.4` in `http://[::ffff:1.2.3.4]/foo?bar#baz`.
    Ipv6Address,

    /// This is the `IPvFuture` host name in the URI, such as
    /// `v7.aB` in `http://[v7.aB]/foo?bar#baz`.
    IpvFuture,

    /// This is the path of the URI, such as `/foo` in
    /// `http://www.example.com/foo?bar#baz`.
    Path,

    /// This is the query of the URI, such as `?bar` in
    /// `http://www.example.com/foo?bar#baz`.
    Query,

    /// This is the scheme of the URI, such as `http` in
    /// `http://www.example.com/foo?bar#baz`.
    Scheme,

    /// This is the scheme of the URI, such as `nobody` in
    /// `http://nobody@www.example.com/foo?bar#baz`.
    Userinfo,
}

impl std::fmt::Display for Context {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Context::Fragment => write!(f, "fragment"),
            Context::Host => write!(f, "host"),
            Context::Ipv4Address => write!(f, "IPv4 address"),
            Context::Ipv6Address => write!(f, "IPv6 address"),
            Context::IpvFuture => write!(f, "IPvFuture"),
            Context::Path => write!(f, "path"),
            Context::Query => write!(f, "query"),
            Context::Scheme => write!(f, "scheme"),
            Context::Userinfo => write!(f, "user info"),
        }
    }
}
