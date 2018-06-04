# URI

[RFC 3986](https://tools.ietf.org/html/rfc3986) -- Uniform Resource Identifier (URI)

## Example

        foo://example.com:8042/over/there?name=ferret#nose
        \_/   \______________/\_________/ \_________/ \__/
        |           |            |            |        |
    scheme     authority       path        query   fragment
        |   _____________________|__
        / \ /                        \
        urn:example:animal:ferret:nose

## Elements

* `IsRelative` -- flag indicating whether or not the reference is `relative reference` (as opposed to a `URI`)
* `Scheme` -- scheme name, if reference isn't relative
* `UserInfo` -- user information (user name, potentially with authorization information), if any is included
* `Host` -- host name or IP address, if included
* `Port` -- port number, if included
* `HasPort` -- flag indicating whether or not to include port number
* `Path` -- sequence of strings
  * Include absolute path by making first element empty
* `Query` -- query string, if included
* `Fragment` -- fragment string, if included

### Special disambiguation rules

* `path-abempty`: For a `URI` with an `authority` component, the path must be either empty or absolute (first element empty); otherwise, it wouldn't possible to distinguish which characters belong to `host` and which belong to the path's first segment.
* `path-noscheme`: A `relative reference` cannot have a path whose first segment contains a colon (`:`) character; otherwise, its string representation would be ambiguous: it would also match a `URI` whose `scheme` is the part of the first segment of the path before the colon).

### Notes

* An empty string property indicates that it isn't included in the reference.
* A zero port number is still valid syntactically, so we need the separate `HasPort` property to indicate whether or not to include `Port`.
* The `authority` component is only included if `Host` is not empty, regardless of whether or not `UserInfo` is empty, and regardless of `HasPort`.
* If the first element of `Path` is an empty string, it indicates that the path is absolute.

## Syntax

    URI-reference = URI / relative-ref

    URI           = scheme ":" hier-part [ "?" query ] [ "#" fragment ]
    absolute-URI  = scheme ":" hier-part [ "?" query ]
    relative-ref  = relative-part [ "?" query ] [ "#" fragment ]

    relative-part = "//" authority path-abempty
                  / path-absolute
                  / path-noscheme
                  / path-empty
    scheme        = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )
    hier-part     = "//" authority path-abempty
                  / path-absolute
                  / path-rootless
                  / path-empty
    query         = *( pchar / "/" / "?" )
    fragment      = *( pchar / "/" / "?" )

    authority     = [ userinfo "@" ] host [ ":" port ]
    path          = path-abempty    ; begins with "/" or is empty
                  / path-absolute   ; begins with "/" but not "//"
                  / path-noscheme   ; begins with a non-colon segment
                  / path-rootless   ; begins with a segment
                  / path-empty      ; zero characters
    path-abempty  = *( "/" segment )
    path-absolute = "/" [ segment-nz *( "/" segment ) ]
    path-noscheme = segment-nz-nc *( "/" segment )
    path-rootless = segment-nz *( "/" segment )
    path-empty    = 0<pchar>
    pchar         = unreserved / pct-encoded / sub-delims / ":" / "@"

    userinfo      = *( unreserved / pct-encoded / sub-delims / ":" )
    host          = IP-literal / IPv4address / reg-name
    port          = *DIGIT
    segment       = *pchar
    segment-nz    = 1*pchar
    segment-nz-nc = 1*( unreserved / pct-encoded / sub-delims / "@" )
                  ; non-zero-length segment without any colon ":"

    IP-literal    = "[" ( IPv6address / IPvFuture  ) "]"
    IPv4address   = dec-octet "." dec-octet "." dec-octet "." dec-octet
    reg-name      = *( unreserved / pct-encoded / sub-delims )

    IPv6address   =                            6( h16 ":" ) ls32
                  /                       "::" 5( h16 ":" ) ls32
                  / [               h16 ] "::" 4( h16 ":" ) ls32
                  / [ *1( h16 ":" ) h16 ] "::" 3( h16 ":" ) ls32
                  / [ *2( h16 ":" ) h16 ] "::" 2( h16 ":" ) ls32
                  / [ *3( h16 ":" ) h16 ] "::"    h16 ":"   ls32
                  / [ *4( h16 ":" ) h16 ] "::"              ls32
                  / [ *5( h16 ":" ) h16 ] "::"              h16
                  / [ *6( h16 ":" ) h16 ] "::"
    IPvFuture     = "v" 1*HEXDIG "." 1*( unreserved / sub-delims / ":" )
    dec-octet     = DIGIT                 ; 0-9
                  / %x31-39 DIGIT         ; 10-99
                  / "1" 2DIGIT            ; 100-199
                  / "2" %x30-34 DIGIT     ; 200-249
                  / "25" %x30-35          ; 250-255

    ls32          = ( h16 ":" h16 ) / IPv4address
                  ; least-significant 32 bits of address
    h16           = 1*4HEXDIG
                  ; 16 bits of address represented in hexadecimal

    pct-encoded   = "%" HEXDIG HEXDIG
    unreserved    = ALPHA / DIGIT / "-" / "." / "_" / "~"
    reserved      = gen-delims / sub-delims
    gen-delims    = ":" / "/" / "?" / "#" / "[" / "]" / "@"
    sub-delims    = "!" / "$" / "&" / "'" / "(" / ")"
                  / "*" / "+" / "," / ";" / "="
