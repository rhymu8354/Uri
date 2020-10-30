use once_cell::sync::Lazy;
use std::collections::HashSet;

// This is the character set containing just the alphabetic characters
// from the ASCII character set.
pub static ALPHA: Lazy<HashSet<char>> =
    Lazy::new(|| ('a'..='z').chain('A'..='Z').collect());

// This is the character set containing just numbers.
pub static DIGIT: Lazy<HashSet<char>> = Lazy::new(|| ('0'..='9').collect());

// This is the character set containing just the characters allowed
// in a hexadecimal digit.
pub static HEXDIG: Lazy<HashSet<char>> = Lazy::new(|| {
    DIGIT.iter().copied().chain('A'..='F').chain('a'..='f').collect()
});

// This is the character set corresponds to the "unreserved" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986).
pub static UNRESERVED: Lazy<HashSet<char>> = Lazy::new(|| {
    ALPHA
        .iter()
        .chain(DIGIT.iter())
        .chain(['-', '.', '_', '~'].iter())
        .copied()
        .collect()
});

// This is the character set corresponds to the "sub-delims" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986).
pub static SUB_DELIMS: Lazy<HashSet<char>> = Lazy::new(|| {
    ['!', '$', '&', '\'', '(', ')', '*', '+', ',', ';', '=']
        .iter()
        .copied()
        .collect()
});

// This is the character set corresponds to the second part
// of the "scheme" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986).
pub static SCHEME_NOT_FIRST: Lazy<HashSet<char>> = Lazy::new(|| {
    ALPHA
        .iter()
        .chain(DIGIT.iter())
        .chain(['+', '-', '.'].iter())
        .copied()
        .collect()
});

// This is the character set corresponds to the "pchar" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
// leaving out "pct-encoded".
pub static PCHAR_NOT_PCT_ENCODED: Lazy<HashSet<char>> = Lazy::new(|| {
    UNRESERVED
        .iter()
        .chain(SUB_DELIMS.iter())
        .chain([':', '@'].iter())
        .copied()
        .collect()
});

// This is the character set corresponds to the "query" syntax
// and the "fragment" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
// leaving out "pct-encoded".
pub static QUERY_OR_FRAGMENT_NOT_PCT_ENCODED: Lazy<HashSet<char>> =
    Lazy::new(|| {
        PCHAR_NOT_PCT_ENCODED.iter().chain(['/', '?'].iter()).copied().collect()
    });

// This is the character set almost corresponds to the "query" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
// leaving out "pct-encoded", except that '+' is also excluded, because
// for some web services (e.g. AWS S3) a '+' is treated as
// synonymous with a space (' ') and thus gets misinterpreted.
pub static QUERY_NOT_PCT_ENCODED_WITHOUT_PLUS: Lazy<HashSet<char>> =
    Lazy::new(|| {
        UNRESERVED
            .iter()
            .chain(
                [
                    '!', '$', '&', '\'', '(', ')', '*', ',', ';', '=', ':',
                    '@', '/', '?',
                ]
                .iter(),
            )
            .copied()
            .collect()
    });

// This is the character set corresponds to the "userinfo" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
// leaving out "pct-encoded".
pub static USER_INFO_NOT_PCT_ENCODED: Lazy<HashSet<char>> = Lazy::new(|| {
    UNRESERVED
        .iter()
        .chain(SUB_DELIMS.iter())
        .chain([':'].iter())
        .copied()
        .collect()
});

// This is the character set corresponds to the "reg-name" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
// leaving out "pct-encoded".
pub static REG_NAME_NOT_PCT_ENCODED: Lazy<HashSet<char>> =
    Lazy::new(|| UNRESERVED.iter().chain(SUB_DELIMS.iter()).copied().collect());

// This is the character set corresponds to the last part of
// the "IPvFuture" syntax
// specified in RFC 3986 (https://tools.ietf.org/html/rfc3986).
pub static IPV_FUTURE_LAST_PART: Lazy<HashSet<char>> = Lazy::new(|| {
    UNRESERVED
        .iter()
        .chain(SUB_DELIMS.iter())
        .chain([':'].iter())
        .copied()
        .collect()
});
