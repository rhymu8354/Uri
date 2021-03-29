use std::{
    collections::HashSet,
    convert::TryFrom,
};

use super::{
    authority::Authority,
    character_classes::{
        ALPHA,
        PCHAR_NOT_PCT_ENCODED,
        QUERY_NOT_PCT_ENCODED_WITHOUT_PLUS,
        QUERY_OR_FRAGMENT_NOT_PCT_ENCODED,
        SCHEME_NOT_FIRST,
    },
    codec::{
        decode_element,
        encode_element,
    },
    context::Context,
    error::Error,
};

/// This type is used to parse and generate URI strings to and from their
/// various components.  Components are percent-encoded as necessary during
/// generation, and percent encodings are decoded during parsing.
///
/// Since most URI components, once decoded, may include non-UTF8 byte
/// sequences (which are always percent-encoded), getter methods such as
/// [`path`] and [`query`] return byte array [slice] references (`&[u8]`)
/// rather than string or string slice references.  Fallible convenience
/// methods ending in `_to_string`, such as [`path_to_string`] and
/// [`query_to_string`], are provided to convert these to strings.
///
/// The "Authority" part of the Uri is represented by the [`Authority` type].
/// Although the `Uri` type provides [`userinfo`], [`host`], and [`port`]
/// methods for convenience, `Uri` holds these components through the
/// [`Authority` type], which can be accessed via [`authority`] and
/// [`set_authority`].  To set or change the userinfo, host, or port of a
/// `Uri`, construct a new `Authority` value and set it in the `Uri` with
/// [`set_authority`].
///
/// # Examples
///
/// ## Parsing a URI into its components
///
/// ```rust
/// # extern crate rhymuri;
/// use rhymuri::Uri;
///
/// # fn main() -> Result<(), rhymuri::Error> {
/// let uri = Uri::parse("http://www.example.com/foo?bar#baz")?;
/// let authority = uri.authority().unwrap();
/// assert_eq!("www.example.com".as_bytes(), authority.host());
/// assert_eq!(Some("www.example.com"), uri.host_to_string()?.as_deref());
/// assert_eq!("/foo", uri.path_to_string()?);
/// assert_eq!(Some("bar"), uri.query_to_string()?.as_deref());
/// assert_eq!(Some("baz"), uri.fragment_to_string()?.as_deref());
/// # Ok(())
/// # }
/// ```
///
/// Implementations are provided for the [`TryFrom`] trait, so that
/// [`TryFrom::try_from`] or [`TryInto::try_into`] may be used as alternatives
/// to [`parse`].
///
/// ## Generating a URI from its components
///
/// ```rust
/// # extern crate rhymuri;
/// use rhymuri::{
///     Authority,
///     Uri,
/// };
///
/// let mut uri = Uri::default();
/// assert!(uri.set_scheme(String::from("http")).is_ok());
/// let mut authority = Authority::default();
/// authority.set_host("www.example.com");
/// uri.set_authority(Some(authority));
/// uri.set_path_from_str("/foo");
/// uri.set_query(Some("bar".into()));
/// uri.set_fragment(Some("baz".into()));
/// assert_eq!("http://www.example.com/foo?bar#baz", uri.to_string());
/// ```
///
/// [`authority`]: #method.authority
/// [`Authority` type]: struct.Authority.html
/// [`host`]: #method.host
/// [`parse`]: #method.parse
/// [`path`]: #method.path
/// [`path_to_string`]: #method.path_to_string
/// [`port`]: #method.port
/// [`query`]: #method.query
/// [`query_to_string`]: #method.query_to_string
/// [`set_authority`]: #method.set_authority
/// [`userinfo`]: #method.userinfo
/// [slice]: https://doc.rust-lang.org/std/primitive.slice.html
/// [`TryFrom::try_from`]: https://doc.rust-lang.org/std/convert/trait.TryFrom.html#tymethod.try_from
/// [`TryInto::try_into`]: https://doc.rust-lang.org/std/convert/trait.TryInto.html#tymethod.try_into
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Uri {
    scheme: Option<String>,
    authority: Option<Authority>,
    path: Vec<Vec<u8>>,
    query: Option<Vec<u8>>,
    fragment: Option<Vec<u8>>,
}

impl Uri {
    /// Borrow the authority (if any) of the URI.
    #[must_use = "respect mah authoritah"]
    pub fn authority(&self) -> Option<&Authority> {
        self.authority.as_ref()
    }

    fn can_navigate_path_up_one_level<T>(path: T) -> bool
    where
        T: AsRef<[Vec<u8>]>,
    {
        let path = path.as_ref();
        match path.first() {
            // First segment empty means path has leading slash,
            // so we can only navigate up if there are two or more segments.
            Some(segment) if segment.is_empty() => path.len() > 1,

            // Otherwise, we can navigate up as long as there is at least one
            // segment.
            Some(_) => true,
            None => false,
        }
    }

    fn check_scheme<T>(scheme: T) -> Result<T, Error>
    where
        T: AsRef<str>,
    {
        match scheme.as_ref() {
            "" => return Err(Error::EmptyScheme),
            scheme => {
                scheme.chars().enumerate().try_fold((), |_, (i, c)| {
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
                })?
            },
        };
        Ok(scheme)
    }

    /// Determines if the URI contains a relative path rather than an absolute
    /// path.
    #[must_use = "please use the return value kthxbye"]
    pub fn contains_relative_path(&self) -> bool {
        !Self::is_path_absolute(&self.path)
    }

    fn decode_query_or_fragment<T>(
        query_or_fragment: T,
        context: Context,
    ) -> Result<Vec<u8>, Error>
    where
        T: AsRef<str>,
    {
        decode_element(
            query_or_fragment,
            &QUERY_OR_FRAGMENT_NOT_PCT_ENCODED,
            context,
        )
    }

    /// Borrow the fragment (if any) of the URI.
    #[must_use = "A query and a fragment walked into a bar.  Too bad you're ignoring the fragment because it's actually a funny joke."]
    pub fn fragment(&self) -> Option<&[u8]> {
        self.fragment.as_deref()
    }

    /// Convert the fragment (if any) into a string.
    ///
    /// # Errors
    ///
    /// Since fragments may contain non-UTF8 byte sequences, this function may
    /// return [`Error::CannotExpressAsUtf8`][CannotExpressAsUtf8].
    ///
    /// [CannotExpressAsUtf8]: enum.Error.html#variant.CannotExpressAsUtf8
    #[must_use = "use the fragment return value silly programmer"]
    pub fn fragment_to_string(&self) -> Result<Option<String>, Error> {
        self.fragment()
            .map(|fragment| {
                String::from_utf8(fragment.to_vec()).map_err(Into::into)
            })
            .transpose()
    }

    /// Borrow the host portion of the Authority (if any) of the URI.
    #[must_use = "why u no use host return value?"]
    pub fn host(&self) -> Option<&[u8]> {
        self.authority.as_ref().map(Authority::host)
    }

    /// Convert the host portion of the Authority (if any) into a string.
    ///
    /// # Errors
    ///
    /// Since host names may contain non-UTF8 byte sequences, this function may
    /// return [`Error::CannotExpressAsUtf8`][CannotExpressAsUtf8].
    ///
    /// [CannotExpressAsUtf8]: enum.Error.html#variant.CannotExpressAsUtf8
    #[must_use = "I made that host field into a string for you; don't you want it?"]
    pub fn host_to_string(&self) -> Result<Option<String>, Error> {
        self.host()
            .map(|host| String::from_utf8(host.to_vec()).map_err(Into::into))
            .transpose()
    }

    fn is_path_absolute<T>(path: T) -> bool
    where
        T: AsRef<[Vec<u8>]>,
    {
        matches!(path.as_ref(), [segment, ..] if segment.is_empty())
    }

    /// Determines if the URI is a `relative-ref` (relative reference), as
    /// defined in [RFC 3986 section
    /// 4.2](https://tools.ietf.org/html/rfc3986#section-4.2).  A relative
    /// reference has no scheme, but may still have an authority.
    #[must_use = "why would you call an accessor method and not use the return value, silly human"]
    pub fn is_relative_reference(&self) -> bool {
        self.scheme.is_none()
    }

    /// Apply the `remove_dot_segments` routine talked about
    /// in [RFC 3986 section
    /// 5.2](https://tools.ietf.org/html/rfc3986#section-5.2) to the path
    /// segments of the URI, in order to normalize the path (apply and remove
    /// "." and ".." segments).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate rhymuri;
    /// use rhymuri::Uri;
    ///
    /// # fn main() -> Result<(), rhymuri::Error> {
    /// let mut uri = Uri::parse("/a/b/c/./../../g")?;
    /// uri.normalize();
    /// assert_eq!("/a/g", uri.path_to_string()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn normalize(&mut self) {
        self.path = Self::normalize_path(&self.path);
    }

    fn normalize_path<T>(original_path: T) -> Vec<Vec<u8>>
    where
        T: AsRef<[Vec<u8>]>,
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
                if !normalized_path.is_empty()
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
            _ => (),
        }
        normalized_path
    }

    /// Interpret the given string as a URI, separating its various components,
    /// returning a `Uri` value containing them.
    ///
    /// # Errors
    ///
    /// There are many ways to screw up a URI string, and this function will
    /// let you know what's up by returning a variant of the
    /// [`Error`](enum.Error.html) type.
    pub fn parse<T>(uri_string: T) -> Result<Self, Error>
    where
        T: AsRef<str>,
    {
        let (scheme, rest) = Self::parse_scheme(uri_string.as_ref())?;
        let path_end = rest.find(&['?', '#'][..]).unwrap_or_else(|| rest.len());
        let authority_and_path_string = &rest[0..path_end];
        let query_and_or_fragment = &rest[path_end..];
        let (authority, path) = Self::split_authority_from_path_and_parse_them(
            authority_and_path_string,
        )?;
        let (fragment, possible_query) =
            Self::parse_fragment(query_and_or_fragment)?;
        let query = Self::parse_query(possible_query)?;
        Ok(Self {
            scheme,
            authority,
            path,
            query,
            fragment,
        })
    }

    fn parse_fragment(
        query_and_or_fragment: &str
    ) -> Result<(Option<Vec<u8>>, &str), Error> {
        if let Some(fragment_delimiter) = query_and_or_fragment.find('#') {
            let fragment = Self::decode_query_or_fragment(
                &query_and_or_fragment[fragment_delimiter + 1..],
                Context::Fragment,
            )?;
            Ok((Some(fragment), &query_and_or_fragment[0..fragment_delimiter]))
        } else {
            Ok((None, query_and_or_fragment))
        }
    }

    fn parse_path<T>(path_string: T) -> Result<Vec<Vec<u8>>, Error>
    where
        T: AsRef<str>,
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

            path_string => path_string
                .split('/')
                .map(|segment| {
                    decode_element(
                        &segment,
                        &PCHAR_NOT_PCT_ENCODED,
                        Context::Path,
                    )
                })
                .collect(),
        }
    }

    fn parse_query<T>(
        query_and_or_fragment: T
    ) -> Result<Option<Vec<u8>>, Error>
    where
        T: AsRef<str>,
    {
        let query_and_or_fragment = query_and_or_fragment.as_ref();
        if query_and_or_fragment.is_empty() {
            Ok(None)
        } else {
            let query = Self::decode_query_or_fragment(
                &query_and_or_fragment[1..],
                Context::Query,
            )?;
            Ok(Some(query))
        }
    }

    fn parse_scheme(uri_string: &str) -> Result<(Option<String>, &str), Error> {
        // Limit our search so we don't scan into the authority
        // or path elements, because these may have the colon
        // character as well, which we might misinterpret
        // as the scheme delimiter.
        let authority_or_path_delimiter_start =
            uri_string.find('/').unwrap_or_else(|| uri_string.len());
        if let Some(scheme_end) =
            &uri_string[0..authority_or_path_delimiter_start].find(':')
        {
            let scheme =
                Self::check_scheme(&uri_string[0..*scheme_end])?.to_lowercase();
            Ok((Some(scheme), &uri_string[*scheme_end + 1..]))
        } else {
            Ok((None, uri_string))
        }
    }

    /// Borrow the path component of the URI.
    ///
    /// The path is represented as a two-dimensional vector:
    /// * the "segments" or pieces of the path between the slashes
    /// * the bytes that make up each segment
    ///
    /// Byte vectors are used instead of strings because segments may contain
    /// non-UTF8 sequences.
    ///
    /// Leading and trailing slashes in the path are special cases represented
    /// by extra empty segments at the beginning and/or end of the path.
    ///
    /// # Examples
    ///
    /// (Note: the examples below show strings, not byte vectors, simply to be
    /// more readable.)
    ///
    /// ```text
    /// "foo/bar"   -> ["foo", "bar"]
    /// "/foo/bar"  -> ["", "foo", "bar"]
    /// "foo/bar/"  -> ["foo", "bar", ""]
    /// "/foo/bar/" -> ["", "foo", "bar", ""]
    /// "/"         -> [""]
    /// ""          -> []
    /// ```
    #[must_use = "you called path() to get the path, so why you no use?"]
    pub fn path(&self) -> &Vec<Vec<u8>> {
        &self.path
    }

    /// Convert the path portion of the URI into a string.
    ///
    /// # Errors
    ///
    /// Since path segments may contain non-UTF8 byte sequences, this function
    /// may return
    /// [`Error::CannotExpressAsUtf8`][CannotExpressAsUtf8].
    ///
    /// [CannotExpressAsUtf8]: enum.Error.html#variant.CannotExpressAsUtf8
    #[must_use = "we went through all that trouble to put the path into a string, and you don't want it?"]
    pub fn path_to_string(&self) -> Result<String, Error> {
        match &*self.path {
            [segment] if segment.is_empty() => Ok("/".to_string()),
            path => Ok(String::from_utf8(path.join(&b"/"[..]))?),
        }
    }

    /// Return a copy of the port (if any) contained in the URI.
    #[must_use = "why did you get the port number and then throw it away?"]
    pub fn port(&self) -> Option<u16> {
        self.authority.as_ref().and_then(Authority::port)
    }

    /// Borrow the query (if any) of the URI.
    #[must_use = "don't you want to know what that query was?"]
    pub fn query(&self) -> Option<&[u8]> {
        self.query.as_deref()
    }

    /// Convert the query (if any) into a string.
    ///
    /// # Errors
    ///
    /// Since queries may contain non-UTF8 byte sequences, this function may
    /// return [`Error::CannotExpressAsUtf8`][CannotExpressAsUtf8].
    ///
    /// [CannotExpressAsUtf8]: enum.Error.html#variant.CannotExpressAsUtf8
    #[must_use = "use the query return value silly programmer"]
    pub fn query_to_string(&self) -> Result<Option<String>, Error> {
        self.query()
            .map(|query| String::from_utf8(query.to_vec()).map_err(Into::into))
            .transpose()
    }

    /// Return a new URI which is the result of applying the given relative
    /// reference to the URI, following the algorithm from [RFC 3986 section
    /// 5.2.2](https://tools.ietf.org/html/rfc3986#section-5.2.2).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate rhymuri;
    /// use rhymuri::Uri;
    ///
    /// # fn main() -> Result<(), rhymuri::Error> {
    /// let base = Uri::parse("http://a/b/c/d;p?q")?;
    /// let relative_reference = Uri::parse("g;x?y#s")?;
    /// let resolved = base.resolve(&relative_reference);
    /// assert_eq!("http://a/b/c/g;x?y#s", resolved.to_string());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use = "why go through all that effort to resolve the URI, when you're not going to use it?!"]
    pub fn resolve(
        &self,
        relative_reference: &Self,
    ) -> Self {
        let (scheme, authority, path, query) =
            if relative_reference.scheme.is_some() {
                (
                    relative_reference.scheme.clone(),
                    relative_reference.authority.clone(),
                    Self::normalize_path(&relative_reference.path),
                    relative_reference.query.clone(),
                )
            } else {
                relative_reference.authority.as_ref().map_or_else(
                    || {
                        let scheme = self.scheme.clone();
                        let authority = self.authority.clone();
                        if relative_reference.path.is_empty() {
                            let path = self.path.clone();
                            let query = if relative_reference.query.is_none() {
                                self.query.clone()
                            } else {
                                relative_reference.query.clone()
                            };
                            (scheme, authority, path, query)
                        } else {
                            let query = relative_reference.query.clone();

                            // RFC describes this as:
                            // "if (R.path starts-with "/") then"
                            if Self::is_path_absolute(&relative_reference.path)
                            {
                                (
                                    scheme,
                                    authority,
                                    relative_reference.path.clone(),
                                    query,
                                )
                            } else {
                                // RFC describes this as:
                                // "T.path = merge(Base.path, R.path);"
                                let mut path = self.path.clone();
                                if path.len() > 1 {
                                    path.pop();
                                }
                                path.extend(
                                    relative_reference.path.iter().cloned(),
                                );
                                (
                                    scheme,
                                    authority,
                                    Self::normalize_path(&path),
                                    query,
                                )
                            }
                        }
                    },
                    |authority| {
                        (
                            self.scheme.clone(),
                            Some(authority.clone()),
                            Self::normalize_path(&relative_reference.path),
                            relative_reference.query.clone(),
                        )
                    },
                )
            };
        Self {
            scheme,
            authority,
            path,
            query,
            fragment: relative_reference.fragment.clone(),
        }
    }

    /// Borrow the scheme (if any) component of the URI.
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

    /// Change the authority of the URI.
    pub fn set_authority<T>(
        &mut self,
        authority: T,
    ) where
        T: Into<Option<Authority>>,
    {
        self.authority = authority.into();
    }

    /// Change the fragment of the URI.
    pub fn set_fragment<T>(
        &mut self,
        fragment: T,
    ) where
        T: Into<Option<Vec<u8>>>,
    {
        self.fragment = fragment.into();
    }

    /// Change the path of the URI.
    ///
    /// Note: See [`path`](#method.path) for special notes about what the
    /// segments of the path mean.
    pub fn set_path<T>(
        &mut self,
        path: T,
    ) where
        T: Into<Vec<Vec<u8>>>,
    {
        self.path = path.into();
    }

    /// Change the path of the URI using a string which is split by its slash
    /// (`/`) characters to determine the path segments.
    ///
    /// Note: See [`path`](#method.path) for special notes about what the
    /// segments of the path mean.
    pub fn set_path_from_str<T>(
        &mut self,
        path: T,
    ) where
        T: AsRef<str>,
    {
        match path.as_ref() {
            "" => self.set_path(vec![]),
            path => self.set_path(
                path.split('/')
                    .map(|segment| segment.as_bytes().to_vec())
                    .collect::<Vec<Vec<u8>>>(),
            ),
        }
    }

    /// Change the query of the URI.
    pub fn set_query<T>(
        &mut self,
        query: T,
    ) where
        T: Into<Option<Vec<u8>>>,
    {
        self.query = query.into();
    }

    /// Change the scheme of the URI.
    ///
    /// # Errors
    ///
    /// The set of characters allowed in the scheme of a URI is limited.
    /// [`Error::IllegalCharacter`](enum.Error.html#variant.IllegalCharacter)
    /// is returned if you try to use a character that isn't allowed.
    pub fn set_scheme<T>(
        &mut self,
        scheme: T,
    ) -> Result<(), Error>
    where
        T: Into<Option<String>>,
    {
        self.scheme = match scheme.into() {
            Some(scheme) => {
                Self::check_scheme(&scheme)?;
                Some(scheme)
            },
            None => None,
        };
        Ok(())
    }

    fn split_authority_from_path_and_parse_them<T>(
        authority_and_path_string: T
    ) -> Result<(Option<Authority>, Vec<Vec<u8>>), Error>
    where
        T: AsRef<str>,
    {
        // Split authority from path.  If there is an authority, parse it.
        let authority_and_path_string = authority_and_path_string.as_ref();
        if let Some(authority_and_path_string) =
            authority_and_path_string.strip_prefix("//")
        {
            // First separate the authority from the path.
            let authority_end = authority_and_path_string
                .find('/')
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

    /// Remove and return the authority portion (if any) of the URI.
    #[must_use]
    pub fn take_authority(&mut self) -> Option<Authority> {
        self.authority.take()
    }

    /// Remove and return the fragment portion (if any) of the URI.
    #[must_use]
    pub fn take_fragment(&mut self) -> Option<Vec<u8>> {
        self.fragment.take()
    }

    /// Remove and return the query portion (if any) of the URI.
    #[must_use]
    pub fn take_query(&mut self) -> Option<Vec<u8>> {
        self.query.take()
    }

    /// Remove and return the scheme portion (if any) of the URI.
    #[must_use]
    pub fn take_scheme(&mut self) -> Option<String> {
        self.scheme.take()
    }

    /// Borrow the userinfo portion (if any) of the Authority (if any) of the
    /// URI.
    ///
    /// Note that you can get `None` if there is either no Authority in the URI
    /// or there is an Authority in the URI but it has no userinfo in it.
    #[must_use = "security breach... security breach... userinfo not used"]
    pub fn userinfo(&self) -> Option<&[u8]> {
        self.authority.as_ref().and_then(Authority::userinfo)
    }

    /// Convert the fragment (if any) into a string.
    ///
    /// # Errors
    ///
    /// Since fragments may contain non-UTF8 byte sequences, this function may
    /// return [`Error::CannotExpressAsUtf8`][CannotExpressAsUtf8].
    ///
    /// [CannotExpressAsUtf8]: enum.Error.html#variant.CannotExpressAsUtf8
    #[must_use = "come on, you intended to use that userinfo return value, didn't you?"]
    pub fn userinfo_to_string(&self) -> Result<Option<String>, Error> {
        self.userinfo()
            .map(|userinfo| {
                String::from_utf8(userinfo.to_vec()).map_err(Into::into)
            })
            .transpose()
    }
}

impl std::fmt::Display for Uri {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        if let Some(scheme) = &self.scheme {
            write!(f, "{}:", scheme)?;
        }
        if let Some(authority) = &self.authority {
            write!(f, "//{}", authority)?;
        }
        // Special case: absolute but otherwise empty path.
        if Self::is_path_absolute(&self.path) && self.path.len() == 1 {
            write!(f, "/")?;
        }
        for (i, segment) in self.path.iter().enumerate() {
            write!(f, "{}", encode_element(segment, &PCHAR_NOT_PCT_ENCODED))?;
            if i + 1 < self.path.len() {
                write!(f, "/")?;
            }
        }
        if let Some(query) = &self.query {
            write!(
                f,
                "?{}",
                encode_element(query, &QUERY_NOT_PCT_ENCODED_WITHOUT_PLUS)
            )?;
        }
        if let Some(fragment) = &self.fragment {
            write!(
                f,
                "#{}",
                encode_element(fragment, &QUERY_OR_FRAGMENT_NOT_PCT_ENCODED)
            )?;
        }
        Ok(())
    }
}

impl TryFrom<&'_ str> for Uri {
    type Error = Error;

    fn try_from(uri_string: &'_ str) -> Result<Self, Self::Error> {
        Uri::parse(uri_string)
    }
}

impl TryFrom<String> for Uri {
    type Error = Error;

    fn try_from(uri_string: String) -> Result<Self, Self::Error> {
        Uri::parse(uri_string)
    }
}

#[cfg(test)]
mod tests {

    use std::convert::TryInto;

    use super::*;

    #[test]
    fn no_scheme() {
        let uri = Uri::parse("foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(None, uri.scheme());
        assert_eq!(&[&b"foo"[..], &b"bar"[..]].to_vec(), uri.path());
        assert_eq!("foo/bar", uri.path_to_string().unwrap());
    }

    #[test]
    fn url() {
        let uri: Result<Uri, Error> =
            "http://www.example.com/foo/bar".try_into();
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some("http"), uri.scheme());
        assert_eq!(Some(&b"www.example.com"[..]), uri.host());
        assert_eq!(
            Some("www.example.com"),
            uri.host_to_string().unwrap().as_deref()
        );
        assert_eq!(uri.path_to_string().unwrap(), "/foo/bar");
    }

    #[test]
    fn urn_default_path_delimiter() {
        let uri = Uri::try_from("urn:book:fantasy:Hobbit");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some("urn"), uri.scheme());
        assert_eq!(None, uri.host());
        assert_eq!(uri.path_to_string().unwrap(), "book:fantasy:Hobbit");
    }

    #[test]
    // NOTE: This lint is disabled because it's triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::from_over_into)]
    fn path_corner_cases() {
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
    fn uri_ends_after_authority() {
        let uri = Uri::parse("http://www.example.com");
        assert!(uri.is_ok());
    }

    #[test]
    // NOTE: This lint is disabled because it's triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::from_over_into)]
    fn relative_vs_non_relative_references() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                is_relative_reference: bool,
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
    // NOTE: This lint is disabled because it's triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::from_over_into)]
    fn relative_vs_non_relative_paths() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                contains_relative_path: bool,
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
                "{}",
                test_index
            );
        }
    }

    #[test]
    // NOTE: These lints are disabled because they're triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::ref_option_ref)]
    #[allow(clippy::from_over_into)]
    fn query_and_fragment_elements() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                host: &'static str,
                query: Option<&'static str>,
                fragment: Option<&'static str>,
            }
        );
        let test_vectors: &[TestVector] = &[
            ("http://www.example.com/", "www.example.com", None, None).into(),
            ("http://example.com?foo", "example.com", Some("foo"), None).into(),
            (
                "http://www.example.com#foo",
                "www.example.com",
                None,
                Some("foo"),
            )
                .into(),
            (
                "http://www.example.com?foo#bar",
                "www.example.com",
                Some("foo"),
                Some("bar"),
            )
                .into(),
            (
                "http://www.example.com?earth?day#bar",
                "www.example.com",
                Some("earth?day"),
                Some("bar"),
            )
                .into(),
            (
                "http://www.example.com/spam?foo#bar",
                "www.example.com",
                Some("foo"),
                Some("bar"),
            )
                .into(),
            ("http://www.example.com/?", "www.example.com", Some(""), None)
                .into(),
        ];
        for (test_index, test_vector) in test_vectors.iter().enumerate() {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(*test_vector.host()),
                uri.host_to_string().unwrap().as_deref()
            );
            assert_eq!(
                *test_vector.query(),
                uri.query_to_string().unwrap().as_deref(),
                "{}",
                test_index
            );
            assert_eq!(
                *test_vector.fragment(),
                uri.fragment_to_string().unwrap().as_deref()
            );
        }
    }

    #[test]
    fn scheme_illegal_characters() {
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
    // NOTE: This lint is disabled because it's triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::from_over_into)]
    fn scheme_barely_legal() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                scheme: &'static str,
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
    fn scheme_mixed_case() {
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
    fn dont_misinterpret_colon_in_other_places_as_scheme_delimiter() {
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
    fn path_illegal_characters() {
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
    // NOTE: This lint is disabled because it's triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::from_over_into)]
    fn path_barely_legal() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                path: Vec<&'static [u8]>,
            }
        );
        let test_vectors: &[TestVector] = &[
            ("/:/foo", vec![&b""[..], &b":"[..], &b"foo"[..]]).into(),
            ("bob@/foo", vec![&b"bob@"[..], &b"foo"[..]]).into(),
            ("hello!", vec![&b"hello!"[..]]).into(),
            ("urn:hello,%20w%6Frld", vec![&b"hello, world"[..]]).into(),
            ("//example.com/foo/(bar)/", vec![
                &b""[..],
                &b"foo"[..],
                &b"(bar)"[..],
                &b""[..],
            ])
                .into(),
        ];
        for test_vector in test_vectors {
            let uri = Uri::parse(test_vector.uri_string());
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(test_vector.path(), uri.path());
        }
    }

    #[test]
    fn query_illegal_characters() {
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
    // NOTE: This lint is disabled because it's triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::from_over_into)]
    fn query_barely_legal() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                query: &'static str,
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
                uri.query_to_string().unwrap().as_deref(),
                "{}",
                test_index
            );
        }
    }

    #[test]
    fn fragment_illegal_characters() {
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
    // NOTE: This lint is disabled because it's triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::from_over_into)]
    fn fragment_barely_legal() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                fragment: &'static str,
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
                uri.fragment_to_string().unwrap().as_deref()
            );
        }
    }

    #[test]
    // NOTE: This lint is disabled because it's triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::from_over_into)]
    fn paths_with_percent_encoded_characters() {
        named_tuple!(
            struct TestVector {
                uri_string: &'static str,
                path_first_segment: &'static [u8],
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
    // NOTE: This lint is disabled because it's triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::from_over_into)]
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
                uri.path_to_string().unwrap(),
                "{}",
                test_vector.uri_string()
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
    // NOTE: This lint is disabled because it's triggered inside the
    // `named_tuple!` macro expansion.
    #[allow(clippy::from_over_into)]
    fn reference_resolution() {
        named_tuple!(
            struct TestVector {
                base_string: &'static str,
                relative_reference_string: &'static str,
                target_string: &'static str,
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
            let relative_reference_uri =
                Uri::parse(test_vector.relative_reference_string()).unwrap();
            let expected_target_uri =
                Uri::parse(test_vector.target_string()).unwrap();
            let actual_target_uri =
                dbg!(base_uri.resolve(&relative_reference_uri));
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
    // NOTE: These lints have to be disabled at the test level because they're
    // triggered inside the `named_tuple!` macro expansion.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::ref_option_ref)]
    #[allow(clippy::from_over_into)]
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
                expected_uri_string: &'static str,
            }
        );
        #[rustfmt::skip]
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
            assert!(uri
                .set_scheme(test_vector.scheme().map(ToString::to_string))
                .is_ok());
            if test_vector.userinfo().is_some()
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
            assert_eq!(*test_vector.expected_uri_string(), uri.to_string());
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
    fn percent_encode_characters_with_two_digits_always() {
        for ci in 0_u8..31_u8 {
            let mut uri = Uri::default();
            uri.set_query(Some(vec![ci]));
            assert_eq!(uri.to_string(), format!("?%{:02X}", ci));
        }
    }

    #[test]
    fn set_illegal_schemes() {
        let test_vectors = ["ab_de", "ab/de", "ab:de", "", "&", "foo&bar"];
        for test_vector in &test_vectors {
            let mut uri = Uri::default();
            assert!(uri.set_scheme(Some((*test_vector).to_string())).is_err());
        }
    }

    #[test]
    fn take_parts() {
        let mut uri =
            Uri::parse("https://www.example.com/foo?bar#baz").unwrap();
        assert_eq!(Some("https"), uri.take_scheme().as_deref());
        assert_eq!("//www.example.com/foo?bar#baz", uri.to_string());
        assert!(matches!(
            uri.take_authority(),
            Some(authority) if authority.host() == b"www.example.com"
        ));
        assert_eq!("/foo?bar#baz", uri.to_string());
        assert!(matches!(uri.take_authority(), None));
        assert_eq!(Some(&b"bar"[..]), uri.take_query().as_deref());
        assert_eq!("/foo#baz", uri.to_string());
        assert_eq!(None, uri.take_query().as_deref());
        assert_eq!(Some(&b"baz"[..]), uri.take_fragment().as_deref());
        assert_eq!("/foo", uri.to_string());
        assert_eq!(None, uri.take_fragment().as_deref());
    }
}
