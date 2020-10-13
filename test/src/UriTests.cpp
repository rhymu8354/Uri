/**
 * @file UriTests.cpp
 *
 * This module contains the unit tests of the Uri::Uri class.
 *
 * © 2018 by Richard Walters
 */

#include <gtest/gtest.h>
#include <stddef.h>
#include <Uri/Uri.hpp>

TEST(UriTests, ParseFromStringNoScheme) {
    Uri::Uri uri;
    ASSERT_TRUE(uri.ParseFromString("foo/bar"));
    ASSERT_EQ("", uri.GetScheme());
    ASSERT_EQ(
        (std::vector< std::string >{
            "foo",
            "bar",
        }),
        uri.GetPath()
    );
}

TEST(UriTests, ParseFromStringUrl) {
    Uri::Uri uri;
    ASSERT_TRUE(uri.ParseFromString("http://www.example.com/foo/bar"));
    ASSERT_EQ("http", uri.GetScheme());
    ASSERT_EQ("www.example.com", uri.GetHost());
    ASSERT_EQ(
        (std::vector< std::string >{
            "",
            "foo",
            "bar",
        }),
        uri.GetPath()
    );
}

TEST(UriTests, ParseFromStringUrnDefaultPathDelimiter) {
    Uri::Uri uri;
    ASSERT_TRUE(uri.ParseFromString("urn:book:fantasy:Hobbit"));
    ASSERT_EQ("urn", uri.GetScheme());
    ASSERT_EQ("", uri.GetHost());
    ASSERT_EQ(
        (std::vector< std::string >{
            "book:fantasy:Hobbit",
        }),
        uri.GetPath()
    );
}

TEST(UriTests, ParseFromStringPathCornerCases) {
    struct TestVector {
        std::string pathIn;
        std::vector< std::string > pathOut;
    };
    const std::vector< TestVector > testVectors{
        {"", {}},
        {"/", {""}},
        {"/foo", {"", "foo"} },
        {"foo/", {"foo", ""} },
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector.pathIn)) << index;
        ASSERT_EQ(testVector.pathOut, uri.GetPath()) << index;
        ++index;
    }
}

TEST(UriTests, ParseFromStringHasAPortNumber) {
    Uri::Uri uri;
    ASSERT_TRUE(uri.ParseFromString("http://www.example.com:8080/foo/bar"));
    ASSERT_EQ("www.example.com", uri.GetHost());
    ASSERT_TRUE(uri.HasPort());
    ASSERT_EQ(8080, uri.GetPort());
}

TEST(UriTests, ParseFromStringDoesNotHaveAPortNumber) {
    Uri::Uri uri;
    ASSERT_TRUE(uri.ParseFromString("http://www.example.com/foo/bar"));
    ASSERT_EQ("www.example.com", uri.GetHost());
    ASSERT_FALSE(uri.HasPort());
}

TEST(UriTests, ParseFromStringTwiceFirstWithPortNumberThenWithout) {
    Uri::Uri uri;
    ASSERT_TRUE(uri.ParseFromString("http://www.example.com:8080/foo/bar"));
    ASSERT_TRUE(uri.ParseFromString("http://www.example.com/foo/bar"));
    ASSERT_FALSE(uri.HasPort());
}

TEST(UriTests, ParseFromStringBadPortNumberPurelyAlphabetic) {
    Uri::Uri uri;
    ASSERT_FALSE(uri.ParseFromString("http://www.example.com:spam/foo/bar"));
}

TEST(UriTests, ParseFromStringBadPortNumberStartsNumericEndsAlphabetic) {
    Uri::Uri uri;
    ASSERT_FALSE(uri.ParseFromString("http://www.example.com:8080spam/foo/bar"));
}

TEST(UriTests, ParseFromStringLargestValidPortNumber) {
    Uri::Uri uri;
    ASSERT_TRUE(uri.ParseFromString("http://www.example.com:65535/foo/bar"));
    ASSERT_TRUE(uri.HasPort());
    ASSERT_EQ(65535, uri.GetPort());
}

TEST(UriTests, ParseFromStringBadPortNumberTooBig) {
    Uri::Uri uri;
    ASSERT_FALSE(uri.ParseFromString("http://www.example.com:65536/foo/bar"));
}

TEST(UriTests, ParseFromStringBadPortNumberNegative) {
    Uri::Uri uri;
    ASSERT_FALSE(uri.ParseFromString("http://www.example.com:-1234/foo/bar"));
}

TEST(UriTests, ParseFromStringEndsAfterAuthority) {
    Uri::Uri uri;
    ASSERT_TRUE(uri.ParseFromString("http://www.example.com"));
}

TEST(UriTests, ParseFromStringRelativeVsNonRelativeReferences) {
    struct TestVector {
        std::string uriString;
        bool isRelativeReference;
    };
    const std::vector< TestVector > testVectors{
        {"http://www.example.com/", false},
        {"http://www.example.com", false},
        {"/", true},
        {"foo", true},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector.uriString)) << index;
        ASSERT_EQ(testVector.isRelativeReference, uri.IsRelativeReference()) << index;
        ++index;
    }
}

TEST(UriTests, ParseFromStringRelativeVsNonRelativePaths) {
    struct TestVector {
        std::string uriString;
        bool containsRelativePath;
    };
    const std::vector< TestVector > testVectors{
        {"http://www.example.com/", false},
        {"http://www.example.com", false},
        {"/", false},
        {"foo", true},

        /*
         * This is only a valid test vector if we understand
         * correctly that an empty string IS a valid
         * "relative reference" URI with an empty path.
         */
        {"", true},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector.uriString)) << index;
        ASSERT_EQ(testVector.containsRelativePath, uri.ContainsRelativePath()) << index;
        ++index;
    }
}

TEST(UriTests, ParseFromStringQueryAndFragmentElements) {
    struct TestVector {
        std::string uriString;
        std::string host;
        std::string query;
        std::string fragment;
    };
    const std::vector< TestVector > testVectors{
        {"http://www.example.com/", "www.example.com", "", ""},
        {"http://example.com?foo", "example.com", "foo", ""},
        {"http://www.example.com#foo", "www.example.com", "", "foo"},
        {"http://www.example.com?foo#bar", "www.example.com", "foo", "bar"},
        {"http://www.example.com?earth?day#bar", "www.example.com", "earth?day", "bar"},
        {"http://www.example.com/spam?foo#bar", "www.example.com", "foo", "bar"},

        /*
         * NOTE: curiously, but we think this is correct, that
         * having a trailing question mark is equivalent to not having
         * any question mark, because in both cases, the query element
         * is empty string.  Perhaps research deeper to see if this is right.
         */
        {"http://www.example.com/?", "www.example.com", "", ""},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector.uriString)) << index;
        ASSERT_EQ(testVector.host, uri.GetHost()) << index;
        ASSERT_EQ(testVector.query, uri.GetQuery()) << index;
        ASSERT_EQ(testVector.fragment, uri.GetFragment()) << index;
        ++index;
    }
}

TEST(UriTests, ParseFromStringUserInfo) {
    struct TestVector {
        std::string uriString;
        std::string userInfo;
    };
    const std::vector< TestVector > testVectors{
        {"http://www.example.com/", ""},
        {"http://joe@www.example.com", "joe"},
        {"http://pepe:feelsbadman@www.example.com", "pepe:feelsbadman"},
        {"//www.example.com", ""},
        {"//bob@www.example.com", "bob"},
        {"/", ""},
        {"foo", ""},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector.uriString)) << index;
        ASSERT_EQ(testVector.userInfo, uri.GetUserInfo()) << index;
        ++index;
    }
}

TEST(UriTests, ParseFromStringTwiceFirstUserInfoThenWithout) {
    Uri::Uri uri;
    ASSERT_TRUE(uri.ParseFromString("http://joe@www.example.com/foo/bar"));
    ASSERT_TRUE(uri.ParseFromString("/foo/bar"));
    ASSERT_TRUE(uri.GetUserInfo().empty());
}

TEST(UriTests, ParseFromStringSchemeIllegalCharacters) {
    const std::vector< std::string > testVectors{
        {"://www.example.com/"},
        {"0://www.example.com/"},
        {"+://www.example.com/"},
        {"@://www.example.com/"},
        {".://www.example.com/"},
        {"h@://www.example.com/"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_FALSE(uri.ParseFromString(testVector)) << index;
        ++index;
    }
}

TEST(UriTests, ParseFromStringSchemeBarelyLegal) {
    struct TestVector {
        std::string uriString;
        std::string scheme;
    };
    const std::vector< TestVector > testVectors{
        {"h://www.example.com/", "h"},
        {"x+://www.example.com/", "x+"},
        {"y-://www.example.com/", "y-"},
        {"z.://www.example.com/", "z."},
        {"aa://www.example.com/", "aa"},
        {"a0://www.example.com/", "a0"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector.uriString)) << index;
        ASSERT_EQ(testVector.scheme, uri.GetScheme());
        ++index;
    }
}

TEST(UriTests, ParseFromStringSchemeMixedCase) {
    const std::vector< std::string > testVectors{
        {"http://www.example.com/"},
        {"hTtp://www.example.com/"},
        {"HTTP://www.example.com/"},
        {"Http://www.example.com/"},
        {"HttP://www.example.com/"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector)) << index;
        ASSERT_EQ("http", uri.GetScheme()) << ">>> Failed for test vector element " << index << " <<<";
        ++index;
    }
}

TEST(UriTests, ParseFromStringHostEndsInDot) {
    Uri::Uri uri;
    ASSERT_TRUE(uri.ParseFromString("http://example.com./foo"));
    ASSERT_EQ("example.com.", uri.GetHost());
}

TEST(UriTests, ParseFromStringUserInfoIllegalCharacters) {
    const std::vector< std::string > testVectors{
        {"//%X@www.example.com/"},
        {"//{@www.example.com/"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_FALSE(uri.ParseFromString(testVector)) << index;
        ++index;
    }
}

TEST(UriTests, ParseFromStringUserInfoBarelyLegal) {
    struct TestVector {
        std::string uriString;
        std::string userInfo;
    };
    const std::vector< TestVector > testVectors{
        {"//%41@www.example.com/", "A"},
        {"//@www.example.com/", ""},
        {"//!@www.example.com/", "!"},
        {"//'@www.example.com/", "'"},
        {"//(@www.example.com/", "("},
        {"//;@www.example.com/", ";"},
        {"http://:@www.example.com/", ":"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector.uriString)) << index;
        ASSERT_EQ(testVector.userInfo, uri.GetUserInfo());
        ++index;
    }
}

TEST(UriTests, ParseFromStringHostIllegalCharacters) {
    const std::vector< std::string > testVectors{
        {"//%X@www.example.com/"},
        {"//@www:example.com/"},
        {"//[vX.:]/"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_FALSE(uri.ParseFromString(testVector)) << index;
        ++index;
    }
}

TEST(UriTests, ParseFromStringHostBarelyLegal) {
    struct TestVector {
        std::string uriString;
        std::string host;
    };
    const std::vector< TestVector > testVectors{
        {"//%41/", "a"},
        {"///", ""},
        {"//!/", "!"},
        {"//'/", "'"},
        {"//(/", "("},
        {"//;/", ";"},
        {"//1.2.3.4/", "1.2.3.4"},
        {"//[v7.:]/", "v7.:"},
        {"//[v7.aB]/", "v7.aB"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector.uriString)) << index;
        ASSERT_EQ(testVector.host, uri.GetHost());
        ++index;
    }
}

TEST(UriTests, ParseFromStringHostMixedCase) {
    const std::vector< std::string > testVectors{
        {"http://www.example.com/"},
        {"http://www.EXAMPLE.com/"},
        {"http://www.exAMple.com/"},
        {"http://www.example.cOM/"},
        {"http://wWw.exampLe.Com/"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector)) << index;
        ASSERT_EQ("www.example.com", uri.GetHost()) << ">>> Failed for test vector element " << index << " <<<";
        ++index;
    }
}

TEST(UriTests, ParseFromStringDontMisinterpretColonInOtherPlacesAsSchemeDelimiter) {
    const std::vector< std::string > testVectors{
        {"//foo:bar@www.example.com/"},
        {"//www.example.com/a:b"},
        {"//www.example.com/foo?a:b"},
        {"//www.example.com/foo#a:b"},
        {"//[v7.:]/"},
        {"/:/foo"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector)) << index;
        ASSERT_TRUE(uri.GetScheme().empty());
        ++index;
    }
}

TEST(UriTests, ParseFromStringPathIllegalCharacters) {
    const std::vector< std::string > testVectors{
        {"http://www.example.com/foo[bar"},
        {"http://www.example.com/]bar"},
        {"http://www.example.com/foo]"},
        {"http://www.example.com/["},
        {"http://www.example.com/abc/foo]"},
        {"http://www.example.com/abc/["},
        {"http://www.example.com/foo]/abc"},
        {"http://www.example.com/[/abc"},
        {"http://www.example.com/foo]/"},
        {"http://www.example.com/[/"},
        {"/foo[bar"},
        {"/]bar"},
        {"/foo]"},
        {"/["},
        {"/abc/foo]"},
        {"/abc/["},
        {"/foo]/abc"},
        {"/[/abc"},
        {"/foo]/"},
        {"/[/"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_FALSE(uri.ParseFromString(testVector)) << index;
        ++index;
    }
}

TEST(UriTests, ParseFromStringPathBarelyLegal) {
    struct TestVector {
        std::string uriString;
        std::vector< std::string > path;
    };
    const std::vector< TestVector > testVectors{
        {"/:/foo", {"", ":", "foo"}},
        {"bob@/foo", {"bob@", "foo"}},
        {"hello!", {"hello!"}},
        {"urn:hello,%20w%6Frld", {"hello, world"}},
        {"//example.com/foo/(bar)/", {"", "foo", "(bar)", ""}},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector.uriString)) << index;
        ASSERT_EQ(testVector.path, uri.GetPath());
        ++index;
    }
}

TEST(UriTests, ParseFromStringQueryIllegalCharacters) {
    const std::vector< std::string > testVectors{
        {"http://www.example.com/?foo[bar"},
        {"http://www.example.com/?]bar"},
        {"http://www.example.com/?foo]"},
        {"http://www.example.com/?["},
        {"http://www.example.com/?abc/foo]"},
        {"http://www.example.com/?abc/["},
        {"http://www.example.com/?foo]/abc"},
        {"http://www.example.com/?[/abc"},
        {"http://www.example.com/?foo]/"},
        {"http://www.example.com/?[/"},
        {"?foo[bar"},
        {"?]bar"},
        {"?foo]"},
        {"?["},
        {"?abc/foo]"},
        {"?abc/["},
        {"?foo]/abc"},
        {"?[/abc"},
        {"?foo]/"},
        {"?[/"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_FALSE(uri.ParseFromString(testVector)) << index;
        ++index;
    }
}

TEST(UriTests, ParseFromStringQueryBarelyLegal) {
    struct TestVector {
        std::string uriString;
        std::string query;
    };
    const std::vector< TestVector > testVectors{
        {"/?:/foo", ":/foo"},
        {"?bob@/foo", "bob@/foo"},
        {"?hello!", "hello!"},
        {"urn:?hello,%20w%6Frld", "hello, world"},
        {"//example.com/foo?(bar)/", "(bar)/"},
        {"http://www.example.com/?foo?bar", "foo?bar" },
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector.uriString)) << index;
        ASSERT_EQ(testVector.query, uri.GetQuery());
        ++index;
    }
}

TEST(UriTests, ParseFromStringFragmentIllegalCharacters) {
    const std::vector< std::string > testVectors{
        {"http://www.example.com/#foo[bar"},
        {"http://www.example.com/#]bar"},
        {"http://www.example.com/#foo]"},
        {"http://www.example.com/#["},
        {"http://www.example.com/#abc/foo]"},
        {"http://www.example.com/#abc/["},
        {"http://www.example.com/#foo]/abc"},
        {"http://www.example.com/#[/abc"},
        {"http://www.example.com/#foo]/"},
        {"http://www.example.com/#[/"},
        {"#foo[bar"},
        {"#]bar"},
        {"#foo]"},
        {"#["},
        {"#abc/foo]"},
        {"#abc/["},
        {"#foo]/abc"},
        {"#[/abc"},
        {"#foo]/"},
        {"#[/"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_FALSE(uri.ParseFromString(testVector)) << index;
        ++index;
    }
}

TEST(UriTests, ParseFromStringFragmentBarelyLegal) {
    struct TestVector {
        std::string uriString;
        std::string fragment;
    };
    const std::vector< TestVector > testVectors{
        {"/#:/foo", ":/foo"},
        {"#bob@/foo", "bob@/foo"},
        {"#hello!", "hello!"},
        {"urn:#hello,%20w%6Frld", "hello, world"},
        {"//example.com/foo#(bar)/", "(bar)/"},
        {"http://www.example.com/#foo?bar", "foo?bar" },
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector.uriString)) << index;
        ASSERT_EQ(testVector.fragment, uri.GetFragment());
        ++index;
    }
}

TEST(UriTests, ParseFromStringPathsWithPercentEncodedCharacters) {
    struct TestVector {
        std::string uriString;
        std::string pathFirstSegment;
    };
    const std::vector< TestVector > testVectors{
        {"%41", "A"},
        {"%4A", "J"},
        {"%4a", "J"},
        {"%bc", "\xbc"},
        {"%Bc", "\xbc"},
        {"%bC", "\xbc"},
        {"%BC", "\xbc"},
        {"%41%42%43", "ABC"},
        {"%41%4A%43%4b", "AJCK"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector.uriString)) << index;
        ASSERT_EQ(testVector.pathFirstSegment, uri.GetPath()[0]);
        ++index;
    }
}

TEST(UriTests, NormalizePath) {
    struct TestVector {
        std::string uriString;
        std::vector< std::string > normalizedPathSegments;
    };
    const std::vector< TestVector > testVectors{
        {"/a/b/c/./../../g", {"", "a", "g"}},
        {"mid/content=5/../6", {"mid", "6"}},
        {"http://example.com/a/../b", {"", "b"}},
        {"http://example.com/../b", {"", "b"}},
        {"http://example.com/a/../b/", {"", "b", ""}},
        {"http://example.com/a/../../b", {"", "b"}},
        {"./a/b", {"a", "b"}},
        {"..", {}},
        {"/", {""}},
        {"a/b/..", {"a", ""}},
        {"a/b/.", {"a", "b", ""}},
        {"a/b/./c", {"a", "b", "c"}},
        {"a/b/./c/", {"a", "b", "c", ""}},
        {"/a/b/..", {"", "a", ""}},
        {"/a/b/.", {"", "a", "b", ""}},
        {"/a/b/./c", {"", "a", "b", "c"}},
        {"/a/b/./c/", {"", "a", "b", "c", ""}},
        {"./a/b/..", {"a", ""}},
        {"./a/b/.", {"a", "b", ""}},
        {"./a/b/./c", {"a", "b", "c"}},
        {"./a/b/./c/", {"a", "b", "c", ""}},
        {"../a/b/..", {"a", ""}},
        {"../a/b/.", {"a", "b", ""}},
        {"../a/b/./c", {"a", "b", "c"}},
        {"../a/b/./c/", {"a", "b", "c", ""}},
        {"../a/b/../c", {"a", "c"}},
        {"../a/b/./../c/", {"a", "c", ""}},
        {"../a/b/./../c", {"a", "c"}},
        {"../a/b/./../c/", {"a", "c", ""}},
        {"../a/b/.././c/", {"a", "c", ""}},
        {"../a/b/.././c", {"a", "c"}},
        {"../a/b/.././c/", {"a", "c", ""}},
        {"/./c/d", {"", "c", "d"}},
        {"/../c/d", {"", "c", "d"}},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        ASSERT_TRUE(uri.ParseFromString(testVector.uriString)) << index;
        uri.NormalizePath();
        ASSERT_EQ(testVector.normalizedPathSegments, uri.GetPath()) << index;
        ++index;
    }
}

TEST(UriTests, ConstructNormalizeAndCompareEquivalentUris) {
    // This was inspired by section 6.2.2
    // of RFC 3986 (https://tools.ietf.org/html/rfc3986).
    Uri::Uri uri1, uri2;
    ASSERT_TRUE(uri1.ParseFromString("example://a/b/c/%7Bfoo%7D"));
    ASSERT_TRUE(uri2.ParseFromString("eXAMPLE://a/./b/../b/%63/%7bfoo%7d"));
    ASSERT_NE(uri1, uri2);
    uri2.NormalizePath();
    ASSERT_EQ(uri1, uri2);
}

TEST(UriTests, ReferenceResolution) {
    struct TestVector {
        std::string baseString;
        std::string relativeReferenceString;
        std::string targetString;
    };
    const std::vector< TestVector > testVectors{
        // These are all taken from section 5.4.1
        // of RFC 3986 (https://tools.ietf.org/html/rfc3986).
        {"http://a/b/c/d;p?q", "g:h", "g:h"},
        {"http://a/b/c/d;p?q", "g", "http://a/b/c/g"},
        {"http://a/b/c/d;p?q", "./g", "http://a/b/c/g"},
        {"http://a/b/c/d;p?q", "g/", "http://a/b/c/g/"},
        {"http://a/b/c/d;p?q", "//g", "http://g"},
        {"http://a/b/c/d;p?q", "?y", "http://a/b/c/d;p?y"},
        {"http://a/b/c/d;p?q", "g?y", "http://a/b/c/g?y"},
        {"http://a/b/c/d;p?q", "#s", "http://a/b/c/d;p?q#s"},
        {"http://a/b/c/d;p?q", "g#s", "http://a/b/c/g#s"},
        {"http://a/b/c/d;p?q", "g?y#s", "http://a/b/c/g?y#s"},
        {"http://a/b/c/d;p?q", ";x", "http://a/b/c/;x"},
        {"http://a/b/c/d;p?q", "g;x", "http://a/b/c/g;x"},
        {"http://a/b/c/d;p?q", "g;x?y#s", "http://a/b/c/g;x?y#s"},
        {"http://a/b/c/d;p?q", "", "http://a/b/c/d;p?q"},
        {"http://a/b/c/d;p?q", ".", "http://a/b/c/"},
        {"http://a/b/c/d;p?q", "./", "http://a/b/c/"},
        {"http://a/b/c/d;p?q", "..", "http://a/b/"},
        {"http://a/b/c/d;p?q", "../", "http://a/b/"},
        {"http://a/b/c/d;p?q", "../g", "http://a/b/g"},
        {"http://a/b/c/d;p?q", "../..", "http://a"},
        {"http://a/b/c/d;p?q", "../../", "http://a"},
        {"http://a/b/c/d;p?q", "../../g", "http://a/g"},

        // Here are some examples of our own.
        {"http://example.com", "foo", "http://example.com/foo"},
        {"http://example.com/", "foo", "http://example.com/foo"},
        {"http://example.com", "foo/", "http://example.com/foo/"},
        {"http://example.com/", "foo/", "http://example.com/foo/"},
        {"http://example.com", "/foo", "http://example.com/foo"},
        {"http://example.com/", "/foo", "http://example.com/foo"},
        {"http://example.com", "/foo/", "http://example.com/foo/"},
        {"http://example.com/", "/foo/", "http://example.com/foo/"},
        {"http://example.com/", "?foo", "http://example.com/?foo"},
        {"http://example.com/", "#foo", "http://example.com/#foo"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri baseUri, relativeReferenceUri, expectedTargetUri;
        ASSERT_TRUE(baseUri.ParseFromString(testVector.baseString));
        ASSERT_TRUE(relativeReferenceUri.ParseFromString(testVector.relativeReferenceString)) << index;
        ASSERT_TRUE(expectedTargetUri.ParseFromString(testVector.targetString)) << index;
        const auto actualTargetUri = baseUri.Resolve(relativeReferenceUri);
        ASSERT_EQ(expectedTargetUri, actualTargetUri) << index;
        ++index;
    }
}

TEST(UriTests, EmptyPathInUriWithAuthorityIsEquivalentToSlashOnlyPath) {
    Uri::Uri uri1, uri2;
    ASSERT_TRUE(uri1.ParseFromString("http://example.com"));
    ASSERT_TRUE(uri2.ParseFromString("http://example.com/"));
    ASSERT_EQ(uri1, uri2);
    ASSERT_TRUE(uri1.ParseFromString("//example.com"));
    ASSERT_TRUE(uri2.ParseFromString("//example.com/"));
    ASSERT_EQ(uri1, uri2);
}

TEST(UriTests, IPv6Address) {
    struct TestVector {
        std::string uriString;
        std::string expectedHost;
        bool isValid;
    };
    const std::vector< TestVector > testVectors{
        // valid
        {"http://[::1]/", "::1", true},
        {"http://[::ffff:1.2.3.4]/", "::ffff:1.2.3.4", true},
        {"http://[2001:db8:85a3:8d3:1319:8a2e:370:7348]/", "2001:db8:85a3:8d3:1319:8a2e:370:7348", true},
        {"http://[2001:db8:85a3:8d3:1319:8a2e:370::]/", "2001:db8:85a3:8d3:1319:8a2e:370::", true},
        {"http://[2001:db8:85a3:8d3:1319:8a2e::1]/", "2001:db8:85a3:8d3:1319:8a2e::1", true},
        {"http://[fFfF::1]", "fFfF::1", true},
        {"http://[1234::1]", "1234::1", true},
        {"http://[fFfF:1:2:3:4:5:6:a]", "fFfF:1:2:3:4:5:6:a", true},
        {"http://[2001:db8:85a3::8a2e:0]/", "2001:db8:85a3::8a2e:0", true},
        {"http://[2001:db8:85a3:8a2e::]/", "2001:db8:85a3:8a2e::", true},

        // invalid
        {"http://[::fFfF::1]", "", false},
        {"http://[::ffff:1.2.x.4]/", "", false},
        {"http://[::ffff:1.2.3.4.8]/", "", false},
        {"http://[::ffff:1.2.3]/", "", false},
        {"http://[::ffff:1.2.3.]/", "", false},
        {"http://[::ffff:1.2.3.256]/", "", false},
        {"http://[::fxff:1.2.3.4]/", "", false},
        {"http://[::ffff:1.2.3.-4]/", "", false},
        {"http://[::ffff:1.2.3. 4]/", "", false},
        {"http://[::ffff:1.2.3.4 ]/", "", false},
        {"http://[::ffff:1.2.3.4/", "", false},
        {"http://::ffff:1.2.3.4]/", "", false},
        {"http://::ffff:a.2.3.4]/", "", false},
        {"http://::ffff:1.a.3.4]/", "", false},
        {"http://[2001:db8:85a3:8d3:1319:8a2e:370:7348:0000]/", "", false},
        {"http://[2001:db8:85a3:8d3:1319:8a2e:370:7348::1]/", "", false},
        {"http://[2001:db8:85a3:8d3:1319:8a2e:370::1]/", "", false},
        {"http://[2001:db8:85a3::8a2e:0:]/", "", false},
        {"http://[2001:db8:85a3::8a2e::]/", "", false},
        {"http://[]/", "", false},
        {"http://[:]/", "", false},
        {"http://[v]/", "", false},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        const bool parseResult = uri.ParseFromString(testVector.uriString);
        ASSERT_EQ(testVector.isValid, parseResult) << index;
        if (parseResult) {
            ASSERT_EQ(testVector.expectedHost, uri.GetHost());
        }
        ++index;
    }
}

TEST(UriTests, IPvFutureAddress) {
    struct TestVector {
        std::string uriString;
        std::string expectedHost;
        bool isValid;
    };
    const std::vector< TestVector > testVectors{
        // valid
        {"http://[v1.x]/", "v1.x", true},
        {"http://[vf.xy]/", "vf.xy", true},
        {"http://[vf.x:y]/", "vf.x:y", true},

        // invalid
        {"http://[vx]/", "", false},
        {"http://[v12]/", "", false},
        {"http://[v1.?]/", "", false},
        {"http://[v1.x?]/", "", false},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        const bool parseResult = uri.ParseFromString(testVector.uriString);
        ASSERT_EQ(testVector.isValid, parseResult) << index;
        if (parseResult) {
            ASSERT_EQ(testVector.expectedHost, uri.GetHost());
        }
        ++index;
    }
}

TEST(UriTests, GenerateString) {
    struct TestVector {
        std::string scheme;
        std::string userinfo;
        std::string host;
        bool hasPort;
        uint16_t port;
        std::vector< std::string > path;
        bool hasQuery;
        std::string query;
        bool hasFragment;
        std::string fragment;
        std::string expectedUriString;
    };
    const std::vector< TestVector > testVectors{
        // general test vectors
        {"http", "bob", "www.example.com", true,  8080, {"", "abc", "def"}, true,  "foobar", true,  "ch2", "http://bob@www.example.com:8080/abc/def?foobar#ch2"},
        {"http", "bob", "www.example.com", true,  0,    {},                 true,  "foobar", true,  "ch2", "http://bob@www.example.com:0?foobar#ch2"},
        {"http", "bob", "www.example.com", true,  0,    {},                 true,  "foobar", true,  "",    "http://bob@www.example.com:0?foobar#"},
        {"",     "",    "example.com",     false, 0,    {},                 true,  "bar",    false, "",    "//example.com?bar"},
        {"",     "",    "example.com",     false, 0,    {},                 true,  ""   ,    false, "",    "//example.com?"},
        {"",     "",    "example.com",     false, 0,    {},                 false, "",       false, "",    "//example.com"},
        {"",     "",    "example.com",     false, 0,    {""},               false, "",       false, "",    "//example.com/"},
        {"",     "",    "example.com",     false, 0,    {"", "xyz"},        false, "",       false, "",    "//example.com/xyz"},
        {"",     "",    "example.com",     false, 0,    {"", "xyz", ""},    false, "",       false, "",    "//example.com/xyz/"},
        {"",     "",    "",                false, 0,    {""},               false, "",       false, "",    "/"},
        {"",     "",    "",                false, 0,    {"", "xyz"},        false, "",       false, "",    "/xyz"},
        {"",     "",    "",                false, 0,    {"", "xyz", ""},    false, "",       false, "",    "/xyz/"},
        {"",     "",    "",                false, 0,    {},                 false, "",       false, "",    ""},
        {"",     "",    "",                false, 0,    {"xyz"},            false, "",       false, "",    "xyz"},
        {"",     "",    "",                false, 0,    {"xyz", ""},        false, "",       false, "",    "xyz/"},
        {"",     "",    "",                false, 0,    {},                 true,  "bar",    false, "",    "?bar"},
        {"http", "",    "",                false, 0,    {},                 true,  "bar",    false, "",    "http:?bar"},
        {"http", "",    "",                false, 0,    {},                 false, "",       false, "",    "http:"},
        {"http", "",    "::1",             false, 0,    {},                 false, "",       false, "",    "http://[::1]"},
        {"http", "",    "::1.2.3.4",       false, 0,    {},                 false, "",       false, "",    "http://[::1.2.3.4]"},
        {"http", "",    "1.2.3.4",         false, 0,    {},                 false, "",       false, "",    "http://1.2.3.4"},
        {"",     "",    "",                false, 0,    {},                 false, "",       false, "",    ""},
        {"http", "bob", "",                false, 0,    {},                 true,  "foobar", false, "",    "http://bob@?foobar"},
        {"",     "bob", "",                false, 0,    {},                 true,  "foobar", false, "",    "//bob@?foobar"},
        {"",     "bob", "",                false, 0,    {},                 false, "",       false, "",    "//bob@"},

        // percent-encoded character test vectors
        {"http", "b b", "www.example.com", true,  8080, {"", "abc", "def"}, true,  "foobar", true,  "ch2", "http://b%20b@www.example.com:8080/abc/def?foobar#ch2"},
        {"http", "bob", "www.e ample.com", true,  8080, {"", "abc", "def"}, true,  "foobar", true,  "ch2", "http://bob@www.e%20ample.com:8080/abc/def?foobar#ch2"},
        {"http", "bob", "www.example.com", true,  8080, {"", "a c", "def"}, true,  "foobar", true,  "ch2", "http://bob@www.example.com:8080/a%20c/def?foobar#ch2"},
        {"http", "bob", "www.example.com", true,  8080, {"", "abc", "def"}, true,  "foo ar", true,  "ch2", "http://bob@www.example.com:8080/abc/def?foo%20ar#ch2"},
        {"http", "bob", "www.example.com", true,  8080, {"", "abc", "def"}, true,  "foobar", true,  "c 2", "http://bob@www.example.com:8080/abc/def?foobar#c%202"},
        {"http", "bob", "ሴ.example.com",   true,  8080, {"", "abc", "def"}, true,  "foobar", false, "",    "http://bob@%E1%88%B4.example.com:8080/abc/def?foobar"},

        // normalization of IPv6 address hex digits
        {"http", "bob", "fFfF::1", true,  8080, {"", "abc", "def"}, true,  "foobar", true,  "c 2", "http://bob@[ffff::1]:8080/abc/def?foobar#c%202"},
    };
    size_t index = 0;
    for (const auto& testVector : testVectors) {
        Uri::Uri uri;
        uri.SetScheme(testVector.scheme);
        uri.SetUserInfo(testVector.userinfo);
        uri.SetHost(testVector.host);
        if (testVector.hasPort) {
            uri.SetPort(testVector.port);
        } else {
            uri.ClearPort();
        }
        uri.SetPath(testVector.path);
        if (testVector.hasQuery) {
            uri.SetQuery(testVector.query);
        } else {
            uri.ClearQuery();
        }
        if (testVector.hasFragment) {
            uri.SetFragment(testVector.fragment);
        } else {
            uri.ClearFragment();
        }
        const auto actualUriString = uri.GenerateString();
        ASSERT_EQ(testVector.expectedUriString, actualUriString) << index;
        ++index;
    }
}

TEST(UriTests, FragmentEmptyButPresent) {
    Uri::Uri uri;
    ASSERT_TRUE(uri.ParseFromString("http://example.com#"));
    ASSERT_TRUE(uri.HasFragment());
    ASSERT_EQ("", uri.GetFragment());
    ASSERT_EQ("http://example.com/#", uri.GenerateString());
    uri.ClearFragment();
    ASSERT_EQ("http://example.com/", uri.GenerateString());
    ASSERT_FALSE(uri.HasFragment());
    ASSERT_TRUE(uri.ParseFromString("http://example.com"));
    ASSERT_FALSE(uri.HasFragment());
    uri.SetFragment("");
    ASSERT_TRUE(uri.HasFragment());
    ASSERT_EQ("", uri.GetFragment());
    ASSERT_EQ("http://example.com/#", uri.GenerateString());
}

TEST(UriTests, QueryEmptyButPresent) {
    Uri::Uri uri;
    ASSERT_TRUE(uri.ParseFromString("http://example.com?"));
    ASSERT_TRUE(uri.HasQuery());
    ASSERT_EQ("", uri.GetQuery());
    ASSERT_EQ("http://example.com/?", uri.GenerateString());
    uri.ClearQuery();
    ASSERT_EQ("http://example.com/", uri.GenerateString());
    ASSERT_FALSE(uri.HasQuery());
    ASSERT_TRUE(uri.ParseFromString("http://example.com"));
    ASSERT_FALSE(uri.HasQuery());
    uri.SetQuery("");
    ASSERT_TRUE(uri.HasQuery());
    ASSERT_EQ("", uri.GetQuery());
    ASSERT_EQ("http://example.com/?", uri.GenerateString());
}

TEST(UriTests, MakeACopy) {
    Uri::Uri uri1;
    (void)uri1.ParseFromString("http://www.example.com/foo.txt");
    Uri::Uri uri2(uri1);
    uri1.SetQuery("bar");
    uri2.SetFragment("page2");
    uri2.SetHost("example.com");
    EXPECT_EQ("http://www.example.com/foo.txt?bar", uri1.GenerateString());
    EXPECT_EQ("http://example.com/foo.txt#page2", uri2.GenerateString());
}

TEST(UriTests, AssignACopy) {
    Uri::Uri uri1;
    (void)uri1.ParseFromString("http://www.example.com/foo.txt");
    Uri::Uri uri2;
    uri2 = uri1;
    uri1.SetQuery("bar");
    uri2.SetFragment("page2");
    uri2.SetHost("example.com");
    EXPECT_EQ("http://www.example.com/foo.txt?bar", uri1.GenerateString());
    EXPECT_EQ("http://example.com/foo.txt#page2", uri2.GenerateString());
}

TEST(UriTests, ClearQuery) {
    Uri::Uri uri;
    (void)uri.ParseFromString("http://www.example.com/?foo=bar");
    uri.ClearQuery();
    EXPECT_EQ("http://www.example.com/", uri.GenerateString());
    EXPECT_FALSE(uri.HasQuery());
}

TEST(UriTests, PercentEncodePlusInQueries) {
    // Although RFC 3986 doesn't say anything about '+', some web services
    // treat it the same as ' ' due to how HTML originally defined how
    // to encode the query portion of a URL
    // (see https://stackoverflow.com/questions/2678551/when-to-encode-space-to-plus-or-20).
    //
    // To avoid issues with these web services, make sure '+' is
    // percent-encoded in a URI when the URI is encoded.
    Uri::Uri uri;
    uri.SetQuery("foo+bar");
    EXPECT_EQ("?foo%2Bbar", uri.GenerateString());
}
