use std::collections::HashSet;

use super::authority::Authority;
use super::codec::{decode_element, encode_element};
use super::context::Context;
use super::error::Error;
use super::character_classes::{
    ALPHA,
    SCHEME_NOT_FIRST,
    PCHAR_NOT_PCT_ENCODED,
    QUERY_OR_FRAGMENT_NOT_PCT_ENCODED,
    QUERY_NOT_PCT_ENCODED_WITHOUT_PLUS,
};

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

    fn can_navigate_path_up_one_level<T>(path: T) -> bool
        where T: AsRef<[Vec<u8>]>
    {
        let path = path.as_ref();
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

    fn check_scheme<T>(scheme: T) -> Result<T, Error>
        where T: AsRef<str>
    {
        match scheme.as_ref() {
            "" => return Err(Error::EmptyScheme),
            scheme => scheme
                .chars()
                .enumerate()
                .try_fold((), |_, (i, c)| {
                    let valid_characters: &HashSet<char> = if i == 0 {
                        &ALPHA
                    } else {
                        &SCHEME_NOT_FIRST
                    };
                    if valid_characters.contains(&c) {
                        Ok(())
                    } else {
                        Err(Error::IllegalCharacter(Context::Scheme))
                    }
                })?,
        };
        Ok(scheme)
    }

    #[must_use = "please use the return value kthxbye"]
    pub fn contains_relative_path(&self) -> bool {
        !Self::is_path_absolute(&self.path)
    }

    fn decode_query_or_fragment<T>(
        query_or_fragment: T,
        context: Context,
    ) -> Result<Vec<u8>, Error>
        where T: AsRef<str>
    {
        decode_element(
            query_or_fragment,
            &QUERY_OR_FRAGMENT_NOT_PCT_ENCODED,
            context
        )
    }

    #[must_use = "A query and a fragment walked into a bar.  Too bad you're ignoring the fragment because it's actually a funny joke."]
    pub fn fragment(&self) -> Option<&[u8]> {
        self.fragment.as_deref()
    }

    #[must_use = "use the fragment return value silly programmer"]
    pub fn fragment_as_string(&self) -> Result<Option<String>, Error> {
        self.fragment()
            .map(|fragment| {
                String::from_utf8(fragment.to_vec())
                    .map_err(Into::into)
            })
            .transpose()
    }

    #[must_use = "why u no use host return value?"]
    pub fn host(&self) -> Option<&[u8]> {
        self.authority
            .as_ref()
            .map(Authority::host)
    }

    #[must_use = "I made that host field into a string for you; don't you want it?"]
    pub fn host_as_string(&self) -> Result<Option<String>, Error> {
        self.host()
            .map(|host| {
                String::from_utf8(host.to_vec())
                    .map_err(Into::into)
            })
            .transpose()
    }

    fn is_path_absolute<T>(path: T) -> bool
        where T: AsRef<[Vec<u8>]>
    {
        match path.as_ref() {
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
    fn normalize_path<T>(original_path: T) -> Vec<Vec<u8>>
        where T: AsRef<[Vec<u8>]>
    {
        // Rebuild the path one segment
        // at a time, removing and applying special
        // navigation segments ("." and "..") as we go.
        //
        // The `at_directory_level` variable tracks whether or not
        // the `normalized_path` refers to a directory.
        let mut at_directory_level = false;
        let mut normalized_path = Vec::new();
        for segment in original_path.as_ref() {
            if segment == b"." {
                at_directory_level = true;
            } else if segment == b".." {
                // Remove last path element
                // if we can navigate up a level.
                if
                    !normalized_path.is_empty()
                    && Self::can_navigate_path_up_one_level(&normalized_path)
                {
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

    pub fn parse<T>(uri_string: T) -> Result<Self, Error>
        where T: AsRef<str>
    {
        let (scheme, rest) = Self::parse_scheme(uri_string.as_ref())?;
        let path_end = rest
            .find(&['?', '#'][..])
            .unwrap_or_else(|| rest.len());
        let authority_and_path_string = &rest[0..path_end];
        let query_and_or_fragment = &rest[path_end..];
        let (authority, path) = Self::split_authority_from_path_and_parse_them(authority_and_path_string)?;
        let (fragment, possible_query) = Self::parse_fragment(query_and_or_fragment)?;
        let query = Self::parse_query(possible_query)?;
        Ok(Self{
            scheme,
            authority,
            path,
            query,
            fragment
        })
    }

    fn parse_fragment(query_and_or_fragment: &str) -> Result<(Option<Vec<u8>>, &str), Error> {
        if let Some(fragment_delimiter) = query_and_or_fragment.find('#') {
            let fragment = Self::decode_query_or_fragment(
                &query_and_or_fragment[fragment_delimiter+1..],
                Context::Fragment
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

    fn parse_path<T>(path_string: T) -> Result<Vec<Vec<u8>>, Error>
        where T: AsRef<str>
    {
        match path_string.as_ref() {
            "/" => {
                // Special case of an empty absolute path, which we want to
                // represent as single empty-string element to indicate that it
                // is absolute.
                Ok(vec![vec![]])
            },

            "" => {
                // Special case of an empty relative path, which we want to
                // represent as an empty vector.
                Ok(vec![])
            },

            path_string => {
                path_string
                    .split('/')
                    .map(|segment| {
                        decode_element(
                            &segment,
                            &PCHAR_NOT_PCT_ENCODED,
                            Context::Path
                        )
                    })
                    .collect()
            }
        }
    }

    fn parse_query<T>(query_and_or_fragment: T) -> Result<Option<Vec<u8>>, Error>
        where T: AsRef<str>
    {
        let query_and_or_fragment = query_and_or_fragment.as_ref();
        if query_and_or_fragment.is_empty() {
            Ok(None)
        } else {
            let query = Self::decode_query_or_fragment(
                &query_and_or_fragment[1..],
                Context::Query
            )?;
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
        match &*self.path {
            [segment] if segment.is_empty() => Ok("/".to_string()),
            path => Ok(
                String::from_utf8(
                    path
                        .join(&b"/"[..])
                )?
            ),
        }
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

    #[must_use = "use the query return value silly programmer"]
    pub fn query_as_string(&self) -> Result<Option<String>, Error> {
        self.query()
            .map(|query| {
                String::from_utf8(query.to_vec())
                    .map_err(Into::into)
            })
            .transpose()
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

    pub fn set_authority<T>(&mut self, authority: T)
        where T: Into<Option<Authority>>
    {
        self.authority = authority.into();
    }

    pub fn set_fragment<T>(&mut self, fragment: T)
        where T: Into<Option<Vec<u8>>>
    {
        self.fragment = fragment.into();
    }

    pub fn set_path<T>(&mut self, path: T)
        where T: Into<Vec<Vec<u8>>>
    {
        self.path = path.into();
    }

    pub fn set_path_from_str<T>(&mut self, path: T)
        where T: AsRef<str>
    {
        match path.as_ref() {
            "" => self.set_path(vec![]),
            path => self.set_path(
                path
                    .split('/')
                    .map(|segment| segment.as_bytes().to_vec())
                    .collect::<Vec<Vec<u8>>>()
            ),
        }
    }

    pub fn set_query<T>(&mut self, query: T)
        where T: Into<Option<Vec<u8>>>
    {
        self.query = query.into();
    }

    pub fn set_scheme<T>(&mut self, scheme: T) -> Result<(), Error>
        where T: Into<Option<String>>
    {
        self.scheme = match scheme.into() {
            Some(scheme) => {
                Self::check_scheme(&scheme)?;
                Some(scheme)
            }
            None => None,
        };
        Ok(())
    }

    fn split_authority_from_path_and_parse_them<T>(
        authority_and_path_string: T
    ) -> Result<(Option<Authority>, Vec<Vec<u8>>), Error>
        where T: AsRef<str>
    {
        // Split authority from path.  If there is an authority, parse it.
        let authority_and_path_string = authority_and_path_string.as_ref();
        if authority_and_path_string.starts_with("//") {
            // Strip off authority marker.
            let authority_and_path_string = &authority_and_path_string[2..];

            // First separate the authority from the path.
            let authority_end = authority_and_path_string.find('/')
                .unwrap_or_else(|| authority_and_path_string.len());
            let authority_string = &authority_and_path_string[0..authority_end];
            let path_string = &authority_and_path_string[authority_end..];

            // Parse the elements inside the authority string.
            let authority = Authority::parse(authority_string)?;
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

    #[must_use = "come on, you intended to use that userinfo return value, didn't you?"]
    pub fn userinfo_as_string(&self) -> Result<Option<String>, Error> {
        self.userinfo()
            .map(|userinfo| {
                String::from_utf8(userinfo.to_vec())
                    .map_err(Into::into)
            })
            .transpose()
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
        if
            Self::is_path_absolute(&self.path)
            && self.path.len() == 1
        {
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parse_from_string_no_scheme() {
        let uri = Uri::parse("foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(None, uri.scheme());
        assert_eq!(&[&b"foo"[..], &b"bar"[..]].to_vec(), uri.path());
        assert_eq!("foo/bar", uri.path_as_string().unwrap());
    }

    #[test]
    fn parse_from_string_url() {
        let uri = Uri::parse("http://www.example.com/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some("http"), uri.scheme());
        assert_eq!(Some(&b"www.example.com"[..]), uri.host());
        assert_eq!(Some("www.example.com"), uri.host_as_string().unwrap().as_deref());
        assert_eq!(uri.path_as_string().unwrap(), "/foo/bar");
    }

    #[test]
    fn parse_from_string_urn_default_path_delimiter() {
        let uri = Uri::parse("urn:book:fantasy:Hobbit");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some("urn"), uri.scheme());
        assert_eq!(None, uri.host());
        assert_eq!(uri.path_as_string().unwrap(), "book:fantasy:Hobbit");
    }

    #[test]
    fn parse_from_string_path_corner_cases() {
        named_tuple!(
            struct TestVector {
                path_in: &'static str,
                path_out: Vec<&'static [u8]>,
            }
        );
        let test_vectors: &[TestVector] = &[
            ("", vec![]).into(),
            ("/", vec![&b""[..]]).into(),
            ("/foo", vec![&b""[..], &b"foo"[..]]).into(),
            ("foo/", vec![&b"foo"[..], &b""[..]]).into(),
        ];
        for test_vector in test_vectors {
            let uri = Uri::parse(test_vector.path_in());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(test_vector.path_out(), uri.path());
        }
    }

    #[test]
    fn parse_from_string_has_a_non_empty_port_number() {
        let uri = Uri::parse("http://www.example.com:8080/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some("www.example.com"), uri.host_as_string().unwrap().as_deref());
        assert_eq!(Some(8080), uri.port());
    }

    #[test]
    fn parse_from_string_has_an_empty_port_number() {
        let uri = Uri::parse("http://www.example.com:/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some("www.example.com"), uri.host_as_string().unwrap().as_deref());
        assert_eq!(None, uri.port());
    }

    #[test]
    fn parse_from_string_does_not_have_a_port_number() {
        let uri = Uri::parse("http://www.example.com/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some("www.example.com"), uri.host_as_string().unwrap().as_deref());
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
        assert!(matches!(uri, Err(Error::IllegalPortNumber(_))));
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
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                is_relative_reference: bool
            }
        );
        let test_vectors: &[TestVector] = &[
            ("http://www.example.com/", false).into(),
            ("http://www.example.com", false).into(),
            ("/", true).into(),
            ("foo", true).into(),
        ];
        for test_vector in test_vectors {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                *test_vector.is_relative_reference(),
                uri.is_relative_reference()
            );
        }
    }

    #[test]
    fn parse_from_string_relative_vs_non_relative_paths() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                contains_relative_path: bool
            }
        );
        let test_vectors: &[TestVector] = &[
            ("http://www.example.com/", false).into(),
            ("http://www.example.com", false).into(),
            ("/", false).into(),
            ("foo", true).into(),

            // This is only a valid test vector if we understand
            // correctly that an empty string IS a valid
            // "relative reference" URI with an empty path.
            ("", true).into(),
        ];
        for (test_index, test_vector) in test_vectors.iter().enumerate() {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                *test_vector.contains_relative_path(),
                uri.contains_relative_path(),
                "{}", test_index
            );
        }
    }

    #[test]
    fn parse_from_string_query_and_fragment_elements() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                host: &'static str,
                query: Option<&'static str>,
                fragment: Option<&'static str>
            }
        );
        let test_vectors: &[TestVector] = &[
            ("http://www.example.com/", "www.example.com", None, None).into(),
            ("http://example.com?foo", "example.com", Some("foo"), None).into(),
            ("http://www.example.com#foo", "www.example.com", None, Some("foo")).into(),
            ("http://www.example.com?foo#bar", "www.example.com", Some("foo"), Some("bar")).into(),
            ("http://www.example.com?earth?day#bar", "www.example.com", Some("earth?day"), Some("bar")).into(),
            ("http://www.example.com/spam?foo#bar", "www.example.com", Some("foo"), Some("bar")).into(),
            ("http://www.example.com/?", "www.example.com", Some(""), None).into(),
        ];
        for (test_index, test_vector) in test_vectors.iter().enumerate() {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(Some(*test_vector.host()), uri.host_as_string().unwrap().as_deref());
            assert_eq!(
                *test_vector.query(),
                uri.query_as_string().unwrap().as_deref(),
                "{}", test_index
            );
            assert_eq!(
                *test_vector.fragment(),
                uri.fragment_as_string().unwrap().as_deref()
            );
        }
    }

    #[test]
    fn parse_from_string_user_info() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                userinfo: Option<&'static str>,
            }
        );
        let test_vectors: &[TestVector] = &[
            ("http://www.example.com/", None).into(),
            ("http://joe@www.example.com", Some("joe")).into(),
            ("http://pepe:feelsbadman@www.example.com", Some("pepe:feelsbadman")).into(),
            ("//www.example.com", None).into(),
            ("//bob@www.example.com", Some("bob")).into(),
            ("/", None).into(),
            ("foo", None).into(),
        ];
        for test_vector in test_vectors {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                *test_vector.userinfo(),
                uri.userinfo_as_string().unwrap().as_deref()
            );
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
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                scheme: &'static str
            }
        );
        let test_vectors: &[TestVector] = &[
            ("h://www.example.com/", "h").into(),
            ("x+://www.example.com/", "x+").into(),
            ("y-://www.example.com/", "y-").into(),
            ("z.://www.example.com/", "z.").into(),
            ("aa://www.example.com/", "aa").into(),
            ("a0://www.example.com/", "a0").into(),
        ];
        for test_vector in test_vectors {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(Some(*test_vector.scheme()), uri.scheme());
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
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                userinfo: &'static str
            }
        );
        let test_vectors: &[TestVector] = &[
            ("//%41@www.example.com/", "A").into(),
            ("//@www.example.com/", "").into(),
            ("//!@www.example.com/", "!").into(),
            ("//'@www.example.com/", "'").into(),
            ("//(@www.example.com/", "(").into(),
            ("//;@www.example.com/", ";").into(),
            ("http://:@www.example.com/", ":").into(),
        ];
        for test_vector in test_vectors {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(*test_vector.userinfo()),
                uri.userinfo_as_string().unwrap().as_deref()
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
            let uri = Uri::parse(test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_host_barely_legal() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                host: &'static str
            }
        );
        let test_vectors: &[TestVector] = &[
            ("//%41/", "a").into(),
            ("///", "").into(),
            ("//!/", "!").into(),
            ("//'/", "'").into(),
            ("//(/", "(").into(),
            ("//;/", ";").into(),
            ("//1.2.3.4/", "1.2.3.4").into(),
            ("//[v7.:]/", "v7.:").into(),
            ("//[v7.aB]/", "v7.aB").into(),
        ];
        for test_vector in test_vectors {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(Some(*test_vector.host()), uri.host_as_string().unwrap().as_deref());
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
        let normalized_host = "www.example.com";
        for test_vector in &test_vectors {
            let uri = Uri::parse(*test_vector);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(normalized_host),
                uri.host_as_string().unwrap().as_deref()
            );
        }
    }

    #[test]
    fn parse_from_string_host_ends_in_dot() {
        let uri = Uri::parse("http://example.com./foo");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some("example.com."), uri.host_as_string().unwrap().as_deref());
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
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                path: Vec<&'static [u8]>
            }
        );
        let test_vectors: &[TestVector] = &[
            ("/:/foo", vec![&b""[..], &b":"[..], &b"foo"[..]]).into(),
            ("bob@/foo", vec![&b"bob@"[..], &b"foo"[..]]).into(),
            ("hello!", vec![&b"hello!"[..]]).into(),
            ("urn:hello,%20w%6Frld", vec![&b"hello, world"[..]]).into(),
            ("//example.com/foo/(bar)/", vec![&b""[..], &b"foo"[..], &b"(bar)"[..], &b""[..]]).into(),
        ];
        for test_vector in test_vectors {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(test_vector.path(), uri.path());
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
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                query: &'static str
            }
        );
        let test_vectors: &[TestVector] = &[
            ("/?:/foo", ":/foo").into(),
            ("?bob@/foo", "bob@/foo").into(),
            ("?hello!", "hello!").into(),
            ("urn:?hello,%20w%6Frld", "hello, world").into(),
            ("//example.com/foo?(bar)/", "(bar)/").into(),
            ("http://www.example.com/?foo?bar", "foo?bar").into(),
        ];
        for (test_index, test_vector) in test_vectors.iter().enumerate() {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(*test_vector.query()),
                uri.query_as_string().unwrap().as_deref(),
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
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                fragment: &'static str
            }
        );
        let test_vectors: &[TestVector] = &[
            ("/#:/foo", ":/foo").into(),
            ("#bob@/foo", "bob@/foo").into(),
            ("#hello!", "hello!").into(),
            ("urn:#hello,%20w%6Frld", "hello, world").into(),
            ("//example.com/foo#(bar)/", "(bar)/").into(),
            ("http://www.example.com/#foo?bar", "foo?bar").into(),
        ];
        for test_vector in test_vectors {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(*test_vector.fragment()),
                uri.fragment_as_string().unwrap().as_deref()
            );
        }
    }

    #[test]
    fn parse_from_string_paths_with_percent_encoded_characters() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                path_first_segment: &'static [u8]
            }
        );
        let test_vectors: &[TestVector] = &[
            ("%41", &b"A"[..]).into(),
            ("%4A", &b"J"[..]).into(),
            ("%4a", &b"J"[..]).into(),
            ("%bc", &b"\xBC"[..]).into(),
            ("%Bc", &b"\xBC"[..]).into(),
            ("%bC", &b"\xBC"[..]).into(),
            ("%BC", &b"\xBC"[..]).into(),
            ("%41%42%43", &b"ABC"[..]).into(),
            ("%41%4A%43%4b", &b"AJCK"[..]).into(),
        ];
        for test_vector in test_vectors {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                test_vector.path_first_segment(),
                uri.path().first().unwrap()
            );
        }
    }

    #[test]
    fn normalize_path() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                normalized_path: &'static str,
            }
        );
        let test_vectors: &[TestVector] = &[
            ("/a/b/c/./../../g", "/a/g").into(),
            ("mid/content=5/../6", "mid/6").into(),
            ("http://example.com/a/../b", "/b").into(),
            ("http://example.com/../b", "/b").into(),
            ("http://example.com/a/../b/", "/b/").into(),
            ("http://example.com/a/../../b", "/b").into(),
            ("./a/b", "a/b").into(),
            ("", "").into(),
            (".", "").into(),
            ("./", "").into(),
            ("..", "").into(),
            ("../", "").into(),
            ("/", "/").into(),
            ("a/b/..", "a/").into(),
            ("a/b/../", "a/").into(),
            ("a/b/.", "a/b/").into(),
            ("a/b/./", "a/b/").into(),
            ("a/b/./c", "a/b/c").into(),
            ("a/b/./c/", "a/b/c/").into(),
            ("/a/b/..", "/a/").into(),
            ("/a/b/.", "/a/b/").into(),
            ("/a/b/./c", "/a/b/c").into(),
            ("/a/b/./c/", "/a/b/c/").into(),
            ("./a/b/..", "a/").into(),
            ("./a/b/.", "a/b/").into(),
            ("./a/b/./c", "a/b/c").into(),
            ("./a/b/./c/", "a/b/c/").into(),
            ("../a/b/..", "a/").into(),
            ("../a/b/.", "a/b/").into(),
            ("../a/b/./c", "a/b/c").into(),
            ("../a/b/./c/", "a/b/c/").into(),
            ("../a/b/../c", "a/c").into(),
            ("../a/b/./../c/", "a/c/").into(),
            ("../a/b/./../c", "a/c").into(),
            ("../a/b/./../c/", "a/c/").into(),
            ("../a/b/.././c/", "a/c/").into(),
            ("../a/b/.././c", "a/c").into(),
            ("../a/b/.././c/", "a/c/").into(),
            ("/./c/d", "/c/d").into(),
            ("/../c/d", "/c/d").into(),
        ];
        for test_vector in test_vectors.iter() {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let mut uri = uri.unwrap();
            uri.normalize();
            assert_eq!(
                *test_vector.normalized_path(),
                uri.path_as_string().unwrap(),
                "{}", test_vector.uri_string()
            );
        }
    }

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
        named_tuple!(
            struct TestVector {
                base_string: &'static str,
                relative_reference_string: &'static str,
                target_string: &'static str
            }
        );
        let test_vectors: &[TestVector] = &[
            // These are all taken from section 5.4.1
            // of RFC 3986 (https://tools.ietf.org/html/rfc3986).
            ("http://a/b/c/d;p?q", "g:h", "g:h").into(),
            ("http://a/b/c/d;p?q", "g", "http://a/b/c/g").into(),
            ("http://a/b/c/d;p?q", "./g", "http://a/b/c/g").into(),
            ("http://a/b/c/d;p?q", "g/", "http://a/b/c/g/").into(),
            ("http://a/b/c/d;p?q", "//g", "http://g").into(),
            ("http://a/b/c/d;p?q", "?y", "http://a/b/c/d;p?y").into(),
            ("http://a/b/c/d;p?q", "g?y", "http://a/b/c/g?y").into(),
            ("http://a/b/c/d;p?q", "#s", "http://a/b/c/d;p?q#s").into(),
            ("http://a/b/c/d;p?q", "g#s", "http://a/b/c/g#s").into(),
            ("http://a/b/c/d;p?q", "g?y#s", "http://a/b/c/g?y#s").into(),
            ("http://a/b/c/d;p?q", ";x", "http://a/b/c/;x").into(),
            ("http://a/b/c/d;p?q", "g;x", "http://a/b/c/g;x").into(),
            ("http://a/b/c/d;p?q", "g;x?y#s", "http://a/b/c/g;x?y#s").into(),
            ("http://a/b/c/d;p?q", "", "http://a/b/c/d;p?q").into(),
            ("http://a/b/c/d;p?q", ".", "http://a/b/c/").into(),
            ("http://a/b/c/d;p?q", "./", "http://a/b/c/").into(),
            ("http://a/b/c/d;p?q", "..", "http://a/b/").into(),
            ("http://a/b/c/d;p?q", "../", "http://a/b/").into(),
            ("http://a/b/c/d;p?q", "../g", "http://a/b/g").into(),
            ("http://a/b/c/d;p?q", "../..", "http://a").into(),
            ("http://a/b/c/d;p?q", "../../", "http://a").into(),
            ("http://a/b/c/d;p?q", "../../g", "http://a/g").into(),

            // Here are some examples of our own.
            ("http://example.com", "foo", "http://example.com/foo").into(),
            ("http://example.com/", "foo", "http://example.com/foo").into(),
            ("http://example.com", "foo/", "http://example.com/foo/").into(),
            ("http://example.com/", "foo/", "http://example.com/foo/").into(),
            ("http://example.com", "/foo", "http://example.com/foo").into(),
            ("http://example.com/", "/foo", "http://example.com/foo").into(),
            ("http://example.com", "/foo/", "http://example.com/foo/").into(),
            ("http://example.com/", "/foo/", "http://example.com/foo/").into(),
            ("http://example.com/", "?foo", "http://example.com/?foo").into(),
            ("http://example.com/", "#foo", "http://example.com/#foo").into(),
        ];
        for test_vector in test_vectors {
            let base_uri = Uri::parse(test_vector.base_string()).unwrap();
            let relative_reference_uri = Uri::parse(test_vector.relative_reference_string()).unwrap();
            let expected_target_uri = Uri::parse(test_vector.target_string()).unwrap();
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
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                expected_host: &'static str,
            }
        );
        let test_vectors: &[TestVector] = &[
            ("http://[::1]/", "::1").into(),
            ("http://[::ffff:1.2.3.4]/", "::ffff:1.2.3.4").into(),
            ("http://[2001:db8:85a3:8d3:1319:8a2e:370:7348]/", "2001:db8:85a3:8d3:1319:8a2e:370:7348").into(),
            ("http://[2001:db8:85a3:8d3:1319:8a2e:370::]/", "2001:db8:85a3:8d3:1319:8a2e:370::").into(),
            ("http://[2001:db8:85a3:8d3:1319:8a2e::1]/", "2001:db8:85a3:8d3:1319:8a2e::1").into(),
            ("http://[fFfF::1]", "fFfF::1").into(),
            ("http://[1234::1]", "1234::1").into(),
            ("http://[fFfF:1:2:3:4:5:6:a]", "fFfF:1:2:3:4:5:6:a").into(),
            ("http://[2001:db8:85a3::8a2e:0]/", "2001:db8:85a3::8a2e:0").into(),
            ("http://[2001:db8:85a3:8a2e::]/", "2001:db8:85a3:8a2e::").into(),
        ];
        for test_vector in test_vectors {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            assert_eq!(
                Some(*test_vector.expected_host()),
                uri.unwrap().host_as_string().unwrap().as_deref()
            );
        }
    }

    #[test]
    fn ipv6_address_bad() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                expected_error: Error,
            }
        );
        let test_vectors: &[TestVector] = &[
            ("http://[::fFfF::1]", Error::TooManyDoubleColons).into(),
            ("http://[::ffff:1.2.x.4]/", Error::IllegalCharacter(Context::Ipv4Address)).into(),
            ("http://[::ffff:1.2.3.4.8]/", Error::TooManyAddressParts).into(),
            ("http://[::ffff:1.2.3]/", Error::TooFewAddressParts).into(),
            ("http://[::ffff:1.2.3.]/", Error::TruncatedHost).into(),
            ("http://[::ffff:1.2.3.256]/", Error::InvalidDecimalOctet).into(),
            ("http://[::fxff:1.2.3.4]/", Error::IllegalCharacter(Context::Ipv6Address)).into(),
            ("http://[::ffff:1.2.3.-4]/", Error::IllegalCharacter(Context::Ipv4Address)).into(),
            ("http://[::ffff:1.2.3. 4]/", Error::IllegalCharacter(Context::Ipv4Address)).into(),
            ("http://[::ffff:1.2.3.4 ]/", Error::IllegalCharacter(Context::Ipv4Address)).into(),
            ("http://[::ffff:1.2.3.4/", Error::TruncatedHost).into(),
            ("http://[2001:db8:85a3:8d3:1319:8a2e:370:7348:0000]/", Error::TooManyAddressParts).into(),
            ("http://[2001:db8:85a3:8d3:1319:8a2e:370:7348::1]/", Error::TooManyAddressParts).into(),
            ("http://[2001:db8:85a3:8d3:1319:8a2e:370::1]/", Error::TooManyAddressParts).into(),
            ("http://[2001:db8:85a3::8a2e:0:]/", Error::TruncatedHost).into(),
            ("http://[2001:db8:85a3::8a2e::]/", Error::TooManyDoubleColons).into(),
            ("http://[]/", Error::TooFewAddressParts).into(),
            ("http://[:]/", Error::TruncatedHost).into(),
            ("http://[v]/", Error::TruncatedHost).into(),
        ];
        for test_vector in test_vectors {
            let uri = Uri::parse(test_vector.uri_string());
            assert_eq!(
                *test_vector.expected_error(),
                uri.unwrap_err(),
                "{}",
                test_vector.uri_string()
            );
        }

        // This is a special case because std::num doesn't trust that we're
        // good enough to make our own ParseIntError values.  FeelsBadMan
        let uri = Uri::parse("http://::ffff:1.2.3.4]/");
        assert!(matches!(uri, Err(Error::IllegalPortNumber(_))));
    }

    #[test]
    // NOTE: `clippy::too_many_arguments` lint has to be disabled at the
    // test level because it's triggered inside the `named_tuple!` macro
    // expansion.
    #[allow(clippy::too_many_arguments)]
    fn generate_string() {
        named_tuple!(
            struct TestVector {
                scheme: Option<&'static str>,
                userinfo: Option<&'static str>,
                host: Option<&'static str>,
                port: Option<u16>,
                path: &'static str,
                query: Option<&'static str>,
                fragment: Option<&'static str>,
                expected_uri_string: &'static str
            }
        );
        let test_vectors: &[TestVector] = &[
            // general test vectors
            // scheme      userinfo     host                     port        path         query           fragment     expected_uri_string
            (Some("http"), Some("bob"), Some("www.example.com"), Some(8080), "/abc/def",  Some("foobar"), Some("ch2"), "http://bob@www.example.com:8080/abc/def?foobar#ch2").into(),
            (Some("http"), Some("bob"), Some("www.example.com"), Some(0),    "",          Some("foobar"), Some("ch2"), "http://bob@www.example.com:0?foobar#ch2").into(),
            (Some("http"), Some("bob"), Some("www.example.com"), Some(0),    "",          Some("foobar"), Some(""),    "http://bob@www.example.com:0?foobar#").into(),
            (None,         None,        Some("example.com"),     None,       "",          Some("bar"),    None,        "//example.com?bar").into(),
            (None,         None,        Some("example.com"),     None,       "",          Some(""),       None,        "//example.com?").into(),
            (None,         None,        Some("example.com"),     None,       "",          None,           None,        "//example.com").into(),
            (None,         None,        Some("example.com"),     None,       "/",         None,           None,        "//example.com/").into(),
            (None,         None,        Some("example.com"),     None,       "/xyz",      None,           None,        "//example.com/xyz").into(),
            (None,         None,        Some("example.com"),     None,       "/xyz/",     None,           None,        "//example.com/xyz/").into(),
            (None,         None,        None,                    None,       "/",         None,           None,        "/").into(),
            (None,         None,        None,                    None,       "/xyz",      None,           None,        "/xyz").into(),
            (None,         None,        None,                    None,       "/xyz/",     None,           None,        "/xyz/").into(),
            (None,         None,        None,                    None,       "",          None,           None,        "").into(),
            (None,         None,        None,                    None,       "xyz",       None,           None,        "xyz").into(),
            (None,         None,        None,                    None,       "xyz/",      None,           None,        "xyz/").into(),
            (None,         None,        None,                    None,       "",          Some("bar"),    None,        "?bar").into(),
            (Some("http"), None,        None,                    None,       "",          Some("bar"),    None,        "http:?bar").into(),
            (Some("http"), None,        None,                    None,       "",          None,           None,        "http:").into(),
            (Some("http"), None,        Some("::1"),             None,       "",          None,           None,        "http://[::1]").into(),
            (Some("http"), None,        Some("::1.2.3.4"),       None,       "",          None,           None,        "http://[::1.2.3.4]").into(),
            (Some("http"), None,        Some("1.2.3.4"),         None,       "",          None,           None,        "http://1.2.3.4").into(),
            (None,         None,        None,                    None,       "",          None,           None,        "").into(),
            (Some("http"), Some("bob"), None,                    None,       "",          Some("foobar"), None,        "http://bob@?foobar").into(),
            (None,         Some("bob"), None,                    None,       "",          Some("foobar"), None,        "//bob@?foobar").into(),
            (None,         Some("bob"), None,                    None,       "",          None,           None,        "//bob@").into(),

            // percent-encoded character test vectors
            // scheme      userinfo      host                     port        path        query            fragment     expected_uri_string
            (Some("http"), Some("b b"),  Some("www.example.com"), Some(8080), "/abc/def", Some("foobar"),  Some("ch2"), "http://b%20b@www.example.com:8080/abc/def?foobar#ch2").into(),
            (Some("http"), Some("bob"),  Some("www.e ample.com"), Some(8080), "/abc/def", Some("foobar"),  Some("ch2"), "http://bob@www.e%20ample.com:8080/abc/def?foobar#ch2").into(),
            (Some("http"), Some("bob"),  Some("www.example.com"), Some(8080), "/a c/def", Some("foobar"),  Some("ch2"), "http://bob@www.example.com:8080/a%20c/def?foobar#ch2").into(),
            (Some("http"), Some("bob"),  Some("www.example.com"), Some(8080), "/abc/def", Some("foo ar"),  Some("ch2"), "http://bob@www.example.com:8080/abc/def?foo%20ar#ch2").into(),
            (Some("http"), Some("bob"),  Some("www.example.com"), Some(8080), "/abc/def", Some("foobar"),  Some("c 2"), "http://bob@www.example.com:8080/abc/def?foobar#c%202").into(),
            (Some("http"), Some("bob"),  Some("ሴ.example.com"),   Some(8080), "/abc/def", Some("foobar"),  None,        "http://bob@%E1%88%B4.example.com:8080/abc/def?foobar").into(),

            // normalization of IPv6 address hex digits
            // scheme      userinfo     host                   port        path        query           fragment     expected_uri_string
            (Some("http"), Some("bob"), Some("fFfF::1"),       Some(8080), "/abc/def", Some("foobar"), Some("c 2"), "http://bob@[ffff::1]:8080/abc/def?foobar#c%202").into(),
        ];
        for test_vector in test_vectors {
            let mut uri = Uri::default();
            assert!(uri.set_scheme(test_vector.scheme().map(ToString::to_string)).is_ok());
            if
                test_vector.userinfo().is_some()
                || test_vector.host().is_some()
                || test_vector.port().is_some()
            {
                let mut authority = Authority::default();
                authority.set_userinfo(test_vector.userinfo().map(Into::into));
                authority.set_host(test_vector.host().unwrap_or_else(|| ""));
                authority.set_port(*test_vector.port());
                uri.set_authority(Some(authority));
            } else {
                uri.set_authority(None);
            }
            uri.set_path_from_str(test_vector.path());
            uri.set_query(test_vector.query().map(Into::into));
            uri.set_fragment(test_vector.fragment().map(Into::into));
            assert_eq!(
                *test_vector.expected_uri_string(),
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
        uri.set_fragment(Some(vec![]));
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
        uri.set_query(Some(vec![]));
        assert_eq!(Some(&b""[..]), uri.query());
        assert_eq!(uri.to_string(), "http://example.com/?");
    }

    #[test]
    fn make_a_copy() {
        let mut uri1 = Uri::parse("http://www.example.com/foo.txt").unwrap();
        let mut uri2 = uri1.clone();
        uri1.set_query(Some(b"bar".to_vec()));
        uri2.set_fragment(Some(b"page2".to_vec()));
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
        uri.set_query(Some(b"foo+bar".to_vec()));
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
            assert!(uri.set_scheme(Some((*test_vector).to_string())).is_err());
        }
    }

}
