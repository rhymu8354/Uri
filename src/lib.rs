#![warn(clippy::pedantic)]

#[cfg(test)]
mod tests {

    use std::convert::TryFrom;

    #[test]
    fn parse_from_string_no_scheme() {
        let uri = uriparse::URIReference::try_from("foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(None, uri.scheme());

        // FIXME: Due to what looks like a bug in either uriparse
        // or the Rust core (so probably uriparse after all), I can't
        // order the arguments as I would like:
        //
        //assert_eq!("foo/bar", uri.path());
        //
        // This causes a stack overflow.
        // https://github.com/sgodwincs/uriparse-rs/issues/14
        //
        // So for now, we work around the issue by swapping the two arguments:
        assert_eq!(uri.path(), "foo/bar");
    }

    #[test]
    fn parse_from_string_url() {
        let uri = uriparse::URIReference::try_from("http://www.example.com/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some("http"), uri.scheme().map(uriparse::Scheme::as_str));
        assert_eq!(
            Some("www.example.com".to_string()),
            uri.host().map(std::string::ToString::to_string)
        );
        assert_eq!(uri.path(), "/foo/bar");
    }

    #[test]
    fn parse_from_string_urn_default_path_delimiter() {
        let uri = uriparse::URIReference::try_from("urn:book:fantasy:Hobbit");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some("urn"), uri.scheme().map(uriparse::Scheme::as_str));
        assert_eq!(None, uri.host());
        assert_eq!(uri.path(), "book:fantasy:Hobbit");
    }

    #[test]
    fn parse_from_string_path_corner_cases() {
        let test_vectors = [
            "",
            "/",
            "/foo",
            "foo/"
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(uri.path(), test_vector);
        }
    }

    #[test]
    fn parse_from_string_has_a_port_number() {
        let uri = uriparse::URIReference::try_from("http://www.example.com:8080/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(
            Some("www.example.com".to_string()),
            uri.host().map(std::string::ToString::to_string)
        );
        assert_eq!(Some(8080), uri.port());
    }

    #[test]
    fn parse_from_string_does_not_have_a_port_number() {
        let uri = uriparse::URIReference::try_from("http://www.example.com/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(
            Some("www.example.com".to_string()),
            uri.host().map(std::string::ToString::to_string)
        );
        assert_eq!(None, uri.port());
    }

    #[test]
    fn parse_from_string_twice_first_with_port_number_then_without() {
        let uri = uriparse::URIReference::try_from("http://www.example.com:8080/foo/bar");
        assert!(uri.is_ok());
        let uri = uriparse::URIReference::try_from("http://www.example.com/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(None, uri.port());
    }

    #[test]
    fn parse_from_string_bad_port_number_purly_alphabetic() {
        let uri = uriparse::URIReference::try_from("http://www.example.com:spam/foo/bar");
        assert!(uri.is_err());
    }

    #[test]
    fn parse_from_string_bad_port_number_starts_numeric_ends_alphabetic() {
        let uri = uriparse::URIReference::try_from("http://www.example.com:8080spam/foo/bar");
        assert!(uri.is_err());
    }

    #[test]
    fn parse_from_string_largest_valid_port_number() {
        let uri = uriparse::URIReference::try_from("http://www.example.com:65535/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(Some(65535), uri.port());
    }

    #[test]
    fn parse_from_string_bad_port_number_too_big() {
        let uri = uriparse::URIReference::try_from("http://www.example.com:65536/foo/bar");
        assert!(uri.is_err());
    }

    #[test]
    fn parse_from_string_bad_port_number_negative() {
        let uri = uriparse::URIReference::try_from("http://www.example.com:-1234/foo/bar");
        assert!(uri.is_err());
    }

    #[test]
    fn parse_from_string_ends_after_authority() {
        let uri = uriparse::URIReference::try_from("http://www.example.com");
        assert!(uri.is_ok());
    }

    #[test]
    fn parse_from_string_relative_vs_non_relative_references() {
        struct TestVector {
            uri_string: &'static str,
            is_relative_reference: bool
        };
        let test_vectors = [
            TestVector{ uri_string: "http://www.example.com/", is_relative_reference: false },
            TestVector{ uri_string: "http://www.example.com", is_relative_reference: false },
            TestVector{ uri_string: "/", is_relative_reference: true },
            TestVector{ uri_string: "foo", is_relative_reference: true },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(test_vector.is_relative_reference, uri.is_relative_reference());
        }
    }

    #[test]
    fn parse_from_string_relative_vs_non_relative_paths() {
        struct TestVector {
            uri_string: &'static str,
            contains_relative_path: bool
        };
        let test_vectors = [
            TestVector{ uri_string: "http://www.example.com/", contains_relative_path: false },
            TestVector{ uri_string: "http://www.example.com", contains_relative_path: false },
            TestVector{ uri_string: "/", contains_relative_path: false },
            TestVector{ uri_string: "foo", contains_relative_path: true },

            /*
             * This is only a valid test vector if we understand
             * correctly that an empty string IS a valid
             * "relative reference" URI with an empty path.
             */
             TestVector{ uri_string: "", contains_relative_path: true },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(test_vector.contains_relative_path, uri.is_relative_path_reference());
        }
    }

    #[test]
    fn parse_from_string_query_and_fragment_elements() {
        struct TestVector {
            uri_string: &'static str,
            host: &'static str,
            query: Option<&'static str>,
            fragment: Option<&'static str>
        };
        let test_vectors = [
            TestVector{ uri_string: "http://www.example.com/", host: "www.example.com", query: None, fragment: None },
            TestVector{ uri_string: "http://example.com?foo", host: "example.com", query: Some("foo"), fragment: None },
            TestVector{ uri_string: "http://www.example.com#foo", host: "www.example.com", query: None, fragment: Some("foo") },
            TestVector{ uri_string: "http://www.example.com?foo#bar", host: "www.example.com", query: Some("foo"), fragment: Some("bar") },
            TestVector{ uri_string: "http://www.example.com?earth?day#bar", host: "www.example.com", query: Some("earth?day"), fragment: Some("bar") },
            TestVector{ uri_string: "http://www.example.com/spam?foo#bar", host: "www.example.com", query: Some("foo"), fragment: Some("bar" )},
            TestVector{ uri_string: "http://www.example.com/?", host: "www.example.com", query: Some(""), fragment: None },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(test_vector.host.to_string()),
                uri.host().map(std::string::ToString::to_string)
            );
            assert_eq!(
                test_vector.query.map(std::string::ToString::to_string),
                uri.query().map(std::string::ToString::to_string)
            );
            assert_eq!(
                test_vector.fragment.map(std::string::ToString::to_string),
                uri.fragment().map(std::string::ToString::to_string)
            );
        }
    }

    #[test]
    fn parse_from_string_user_info() {
        struct TestVector {
            uri_string: &'static str,
            username: Option<&'static str>,
            password: Option<&'static str>
        };
        let test_vectors = [
            TestVector{ uri_string: "http://www.example.com/", username: None, password: None },
            TestVector{ uri_string: "http://joe@www.example.com", username: Some("joe"), password: None},
            TestVector{ uri_string: "http://pepe:feelsbadman@www.example.com", username: Some("pepe"), password: Some("feelsbadman") },
            TestVector{ uri_string: "//www.example.com", username: None, password: None },
            TestVector{ uri_string: "//bob@www.example.com", username: Some("bob"), password: None },
            TestVector{ uri_string: "/", username: None, password: None },
            TestVector{ uri_string: "foo", username: None, password: None },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                test_vector.username.map(std::string::ToString::to_string),
                uri.username().map(std::string::ToString::to_string)
            );
            assert_eq!(
                test_vector.password.map(std::string::ToString::to_string),
                uri.password().map(std::string::ToString::to_string)
            );
        }
    }

    #[test]
    fn parse_from_string_twice_first_user_info_then_without() {
        let uri = uriparse::URIReference::try_from("http://joe@www.example.com/foo/bar");
        assert!(uri.is_ok());
        let uri = uriparse::URIReference::try_from("/foo/bar");
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert_eq!(None, uri.username());
    }

    // FIXME: the following test has to be completely commented out
    // because of this bug:
    // https://github.com/sgodwincs/uriparse-rs/issues/15
    //
    // Basically, uriparse doesn't correctly reject input which matches
    // the `path-noscheme` syntax rule.  It permits colon (":") characters
    // in path segments everywhere, despite what the second paragraph
    // of section 3.3 of RFC 3896 has to say:  "In addition, a URI reference
    // (Section 4.1) may be a relative-path reference, in which case the
    // first path segment cannot contain a colon (":") character."
    //
    // #[test]
    // fn parse_from_string_scheme_illegal_characters() {
    //     let test_vectors = [
    //         "://www.example.com/",
    //         "0://www.example.com/",
    //         "+://www.example.com/",
    //         "@://www.example.com/",
    //         ".://www.example.com/",
    //         "h@://www.example.com/",
    //     ];
    //     for test_vector in &test_vectors {
    //         let uri = uriparse::URIReference::try_from(*test_vector);
    //         assert!(uri.is_err());
    //     }
    // }

    #[test]
    fn parse_from_string_scheme_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            scheme: &'static str
        };
        let test_vectors = [
            TestVector{ uri_string: "h://www.example.com/", scheme: "h" },
            TestVector{ uri_string: "x+://www.example.com/", scheme: "x+" },
            TestVector{ uri_string: "y-://www.example.com/", scheme: "y-" },
            TestVector{ uri_string: "z.://www.example.com/", scheme: "z." },
            TestVector{ uri_string: "aa://www.example.com/", scheme: "aa" },
            TestVector{ uri_string: "a0://www.example.com/", scheme: "a0" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(test_vector.scheme),
                uri.scheme().map(uriparse::Scheme::as_str)
            );
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
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some("http"),
                uri.scheme().map(uriparse::Scheme::as_str)
            );
        }
    }

    #[test]
    fn parse_from_string_user_info_illegal_characters() {
        let test_vectors = [
            "//%X@www.example.com/",
            "//{@www.example.com/",
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_user_info_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            username: &'static str
        };
        let test_vectors = [
            TestVector{ uri_string: "//%41@www.example.com/", username: "A" },
            TestVector{ uri_string: "//@www.example.com/", username: "" },
            TestVector{ uri_string: "//!@www.example.com/", username: "!" },
            TestVector{ uri_string: "//'@www.example.com/", username: "'" },
            TestVector{ uri_string: "//(@www.example.com/", username: "(" },
            TestVector{ uri_string: "//;@www.example.com/", username: ";" },
            TestVector{ uri_string: "http://:@www.example.com/", username: "" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(uriparse::Username::try_from(test_vector.username).unwrap()),
                uri.username().map(std::clone::Clone::clone)
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
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_host_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            host: &'static str
        };
        let test_vectors = [
            TestVector{ uri_string: "//%41/", host: "a" },
            TestVector{ uri_string: "///", host: "" },
            TestVector{ uri_string: "//!/", host: "!" },
            TestVector{ uri_string: "//'/", host: "'" },
            TestVector{ uri_string: "//(/", host: "(" },
            TestVector{ uri_string: "//;/", host: ";" },
            TestVector{ uri_string: "//1.2.3.4/", host: "1.2.3.4" },

            // FIXME: These two test vectors are commented out because
            // uriparse cannot parse them correctly.  Although they are
            // valid syntax, we get `HostError::AddressMechanismNotSupported`.
            //
            // It would be nice if uriparse would delegate responsibility to
            // handle IPvFuture host syntax, but unfortunately it doesn't.
            //
            // TestVector{ uri_string: "//[v7.:]/", host: "v7.:" },
            // TestVector{ uri_string: "//[v7.aB]/", host: "v7.aB" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(uriparse::Host::try_from(test_vector.host).unwrap()),
                uri.host().map(std::clone::Clone::clone)
            );
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
        let normalized_host = uriparse::Host::try_from("www.example.com").unwrap();
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(&normalized_host),
                uri.host()
            );
        }
    }

    #[test]
    fn parse_from_string_dont_misinterpret_colon_in_other_places_as_scheme_delimiter() {
        let test_vectors = [
            "//foo:bar@www.example.com/",
            "//www.example.com/a:b",
            "//www.example.com/foo?a:b",
            "//www.example.com/foo#a:b",

            // FIXME: This test vector is commented out because
            // uriparse cannot parse it correctly.  Although it is
            // valid syntax, we get `HostError::AddressMechanismNotSupported`.
            //
            // It would be nice if uriparse would delegate responsibility to
            // handle IPvFuture host syntax, but unfortunately it doesn't.
            //
            // "//[v7.:]/",

            "/:/foo",
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(*test_vector);
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
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_path_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            path: &'static str
        };
        let test_vectors = [
            TestVector{ uri_string: "/:/foo", path: "/:/foo" },
            TestVector{ uri_string: "bob@/foo", path: "bob@/foo" },
            TestVector{ uri_string: "hello!", path: "hello!" },

            // NOTE: uriparse does not do percent encoding for us,
            // and SP (space) is not acceptable in a path, even
            // as input to the `try_from` function.
            //
            // FIXME: For this test vector to pass, we have to normalize
            // the path *after* parsing it from the `uri_string`, despite
            // what the `uriparse` documentation says about percent
            // encoding playing no role in equality checking.
            //
            // https://github.com/sgodwincs/uriparse-rs/issues/16
            TestVector{ uri_string: "urn:hello,%20w%6Frld", path: "hello,%20world" },

            TestVector{ uri_string: "//example.com/foo/(bar)/", path: "/foo/(bar)/" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            let mut path = uri.path().clone();
            path.normalize(false);
            assert_eq!(
                uriparse::Path::try_from(test_vector.path).unwrap(),
                path
            );
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
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_query_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            query: &'static str
        };
        let test_vectors = [
            TestVector{ uri_string: "/?:/foo", query: ":/foo" },
            TestVector{ uri_string: "?bob@/foo", query: "bob@/foo" },
            TestVector{ uri_string: "?hello!", query: "hello!" },

            // NOTE: uriparse does not do percent encoding for us,
            // and SP (space) is not acceptable in a query, even
            // as input to the `try_from` function.
            //
            TestVector{ uri_string: "urn:?hello,%20w%6Frld", query: "hello,%20world" },

            TestVector{ uri_string: "//example.com/foo?(bar)/", query: "(bar)/" },
            TestVector{ uri_string: "http://www.example.com/?foo?bar", query: "foo?bar" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(&uriparse::Query::try_from(test_vector.query).unwrap()),
                uri.query()
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
            let uri = uriparse::URIReference::try_from(*test_vector);
            assert!(uri.is_err());
        }
    }

    #[test]
    fn parse_from_string_fragment_barely_legal() {
        struct TestVector {
            uri_string: &'static str,
            fragment: &'static str
        };
        let test_vectors = [
            TestVector{ uri_string: "/#:/foo", fragment: ":/foo" },
            TestVector{ uri_string: "#bob@/foo", fragment: "bob@/foo" },
            TestVector{ uri_string: "#hello!", fragment: "hello!" },

            // NOTE: uriparse does not do percent encoding for us,
            // and SP (space) is not acceptable in a fragment, even
            // as input to the `try_from` function.
            //
            TestVector{ uri_string: "urn:#hello,%20w%6Frld", fragment: "hello,%20world" },

            TestVector{ uri_string: "//example.com/foo#(bar)/", fragment: "(bar)/" },
            TestVector{ uri_string: "http://www.example.com/#foo?bar", fragment: "foo?bar" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            assert_eq!(
                Some(&uriparse::Fragment::try_from(test_vector.fragment).unwrap()),
                uri.fragment()
            );
        }
    }

    #[test]
    fn parse_from_string_paths_with_percent_encoded_characters() {
        struct TestVector {
            uri_string: &'static str,
            path_first_segment: &'static [u8]
        };
        let test_vectors = [
            TestVector{ uri_string: "%41", path_first_segment: b"A" },
            TestVector{ uri_string: "%4A", path_first_segment: b"J" },
            TestVector{ uri_string: "%4a", path_first_segment: b"J" },

            // Note: uriparse refuses to decode the percent encodings
            // of non-ASCII characters, even if they represent valid
            // UTF-8 encodings.  So we have to keep them percent-encoded,
            // unfortunately.
            TestVector{ uri_string: "%bc", path_first_segment: b"%BC" },
            TestVector{ uri_string: "%Bc", path_first_segment: b"%BC" },
            TestVector{ uri_string: "%bC", path_first_segment: b"%BC" },
            TestVector{ uri_string: "%BC", path_first_segment: b"%BC" },

            TestVector{ uri_string: "%41%42%43", path_first_segment: b"ABC" },
            TestVector{ uri_string: "%41%4A%43%4b", path_first_segment: b"AJCK" },
        ];
        for test_vector in &test_vectors {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            let mut segment = uri.path().segments().first().unwrap().clone();
            segment.normalize();
            assert_eq!(
                segment,
                test_vector.path_first_segment
            );
        }
    }

    #[test]
    fn normalize_path() {
        struct TestVector {
            uri_string: &'static str,
            normalized_path_segments: &'static [&'static str],
            is_absolute: bool
        };
        let test_vectors = [
            TestVector{ uri_string: "/a/b/c/./../../g", normalized_path_segments: &["a", "g"], is_absolute: true },
            TestVector{ uri_string: "mid/content=5/../6", normalized_path_segments: &["mid", "6"], is_absolute: false },
            TestVector{ uri_string: "http://example.com/a/../b", normalized_path_segments: &["b"], is_absolute: true },
            TestVector{ uri_string: "http://example.com/../b", normalized_path_segments: &["b"], is_absolute: true },
            TestVector{ uri_string: "http://example.com/a/../b/", normalized_path_segments: &["b", ""], is_absolute: true },
            TestVector{ uri_string: "http://example.com/a/../../b", normalized_path_segments: &["b"], is_absolute: true },
            TestVector{ uri_string: "./a/b", normalized_path_segments: &["a", "b"], is_absolute: false },
            TestVector{ uri_string: "..", normalized_path_segments: &[""], is_absolute: false },
            TestVector{ uri_string: "/", normalized_path_segments: &[""], is_absolute: true },
            TestVector{ uri_string: "a/b/..", normalized_path_segments: &["a", ""], is_absolute: false },
            TestVector{ uri_string: "a/b/.", normalized_path_segments: &["a", "b", ""], is_absolute: false },
            TestVector{ uri_string: "a/b/./c", normalized_path_segments: &["a", "b", "c"], is_absolute: false },
            TestVector{ uri_string: "a/b/./c/", normalized_path_segments: &["a", "b", "c", ""], is_absolute: false },
            TestVector{ uri_string: "/a/b/..", normalized_path_segments: &["a", ""], is_absolute: true },
            TestVector{ uri_string: "/a/b/.", normalized_path_segments: &["a", "b", ""], is_absolute: true },
            TestVector{ uri_string: "/a/b/./c", normalized_path_segments: &["a", "b", "c"], is_absolute: true },
            TestVector{ uri_string: "/a/b/./c/", normalized_path_segments: &["a", "b", "c", ""], is_absolute: true },
            TestVector{ uri_string: "./a/b/..", normalized_path_segments: &["a", ""], is_absolute: false },
            TestVector{ uri_string: "./a/b/.", normalized_path_segments: &["a", "b", ""], is_absolute: false },
            TestVector{ uri_string: "./a/b/./c", normalized_path_segments: &["a", "b", "c"], is_absolute: false },
            TestVector{ uri_string: "./a/b/./c/", normalized_path_segments: &["a", "b", "c", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/..", normalized_path_segments: &["a", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/.", normalized_path_segments: &["a", "b", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/./c", normalized_path_segments: &["a", "b", "c"], is_absolute: false },
            TestVector{ uri_string: "../a/b/./c/", normalized_path_segments: &["a", "b", "c", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/../c", normalized_path_segments: &["a", "c"], is_absolute: false },
            TestVector{ uri_string: "../a/b/./../c/", normalized_path_segments: &["a", "c", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/./../c", normalized_path_segments: &["a", "c"], is_absolute: false },
            TestVector{ uri_string: "../a/b/./../c/", normalized_path_segments: &["a", "c", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/.././c/", normalized_path_segments: &["a", "c", ""], is_absolute: false },
            TestVector{ uri_string: "../a/b/.././c", normalized_path_segments: &["a", "c"], is_absolute: false },
            TestVector{ uri_string: "../a/b/.././c/", normalized_path_segments: &["a", "c", ""], is_absolute: false },
            TestVector{ uri_string: "/./c/d", normalized_path_segments: &["c", "d"], is_absolute: true },
            TestVector{ uri_string: "/../c/d", normalized_path_segments: &["c", "d"], is_absolute: true },
        ];
        for test_vector in test_vectors.iter() {
            let uri = uriparse::URIReference::try_from(test_vector.uri_string);
            assert!(uri.is_ok());
            let uri = uri.unwrap();
            let mut path = uri.path().clone();
            path.normalize(false);
            assert_eq!(
                path.segments(),
                test_vector.normalized_path_segments.iter().map(
                    |segment| uriparse::Segment::try_from(*segment).unwrap()
                ).collect::<Vec<uriparse::Segment>>()
            );
            assert_eq!(test_vector.is_absolute, path.is_absolute());
        }
    }

    #[test]
    fn construct_normalize_and_compare_equivalent_uris() {
        // This was inspired by section 6.2.2
        // of RFC 3986 (https://tools.ietf.org/html/rfc3986).
        let uri1 = uriparse::URIReference::try_from("example://a/b/c/%7Bfoo%7D");
        assert!(uri1.is_ok());
        let uri1 = uri1.unwrap();
        let uri2 = uriparse::URIReference::try_from("eXAMPLE://a/./b/../b/%63/%7bfoo%7d");
        assert!(uri2.is_ok());
        let mut uri2 = uri2.unwrap();
        assert_ne!(uri1, uri2);
        uri2.normalize();
        assert_eq!(uri1, uri2);
    }

    #[test]
    fn reference_resolution() {
        struct TestVector {
            base_string: &'static str,
            relative_reference_string: &'static str,
            target_string: &'static str
        };
        let test_vectors = [
            // These are all taken from section 5.4.1
            // of RFC 3986 (https://tools.ietf.org/html/rfc3986).
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g:h", target_string: "g:h" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g", target_string: "http://a/b/c/g" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "./g", target_string: "http://a/b/c/g" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g/", target_string: "http://a/b/c/g/" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "//g", target_string: "http://g" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "?y", target_string: "http://a/b/c/d;p?y" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g?y", target_string: "http://a/b/c/g?y" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "#s", target_string: "http://a/b/c/d;p?q#s" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g#s", target_string: "http://a/b/c/g#s" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g?y#s", target_string: "http://a/b/c/g?y#s" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: ";x", target_string: "http://a/b/c/;x" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g;x", target_string: "http://a/b/c/g;x" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "g;x?y#s", target_string: "http://a/b/c/g;x?y#s" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "", target_string: "http://a/b/c/d;p?q" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: ".", target_string: "http://a/b/c/" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "./", target_string: "http://a/b/c/" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "..", target_string: "http://a/b/" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "../", target_string: "http://a/b/" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "../g", target_string: "http://a/b/g" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "../..", target_string: "http://a" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "../../", target_string: "http://a" },
            TestVector{ base_string: "http://a/b/c/d;p?q", relative_reference_string: "../../g", target_string: "http://a/g" },

            // Here are some examples of our own.
            TestVector{ base_string: "http://example.com", relative_reference_string: "foo", target_string: "http://example.com/foo" },
            TestVector{ base_string: "http://example.com/", relative_reference_string: "foo", target_string: "http://example.com/foo" },
            TestVector{ base_string: "http://example.com", relative_reference_string: "foo/", target_string: "http://example.com/foo/" },
            TestVector{ base_string: "http://example.com/", relative_reference_string: "foo/", target_string: "http://example.com/foo/" },
            TestVector{ base_string: "http://example.com", relative_reference_string: "/foo", target_string: "http://example.com/foo" },
            TestVector{ base_string: "http://example.com/", relative_reference_string: "/foo", target_string: "http://example.com/foo" },
            TestVector{ base_string: "http://example.com", relative_reference_string: "/foo/", target_string: "http://example.com/foo/" },
            TestVector{ base_string: "http://example.com/", relative_reference_string: "/foo/", target_string: "http://example.com/foo/" },
            TestVector{ base_string: "http://example.com/", relative_reference_string: "?foo", target_string: "http://example.com/?foo" },
            TestVector{ base_string: "http://example.com/", relative_reference_string: "#foo", target_string: "http://example.com/#foo" },
        ];
        for test_vector in &test_vectors {
            let base_uri = uriparse::URI::try_from(test_vector.base_string).unwrap();
            let relative_reference_uri = uriparse::URIReference::try_from(test_vector.relative_reference_string).unwrap();
            let expected_target_uri = uriparse::URI::try_from(test_vector.target_string).unwrap();
            let actual_target_uri = base_uri.resolve(&relative_reference_uri);
            assert_eq!(expected_target_uri, actual_target_uri);
        }
    }

}
