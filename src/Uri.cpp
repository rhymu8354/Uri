/**
 * @file Uri.cpp
 *
 * This module contains the implementation of the Uri::Uri class.
 *
 * © 2018 by Richard Walters
 */

#include "CharacterSet.hpp"
#include "NormalizeCaseInsensitiveString.hpp"
#include "PercentEncodedCharacterDecoder.hpp"

#include <algorithm>
#include <functional>
#include <inttypes.h>
#include <memory>
#include <string>
#include <Uri/Uri.hpp>
#include <vector>

namespace {

    /**
     * This is the character set containing just the alphabetic characters
     * from the ASCII character set.
     */
    const Uri::CharacterSet ALPHA{
        Uri::CharacterSet('a', 'z'),
        Uri::CharacterSet('A', 'Z')
    };

    /**
     * This is the character set containing just numbers.
     */
    const Uri::CharacterSet DIGIT('0', '9');

    /**
     * This is the character set containing just the characters allowed
     * in a hexadecimal digit.
     */
    const Uri::CharacterSet HEXDIG{
        Uri::CharacterSet('0', '9'),
        Uri::CharacterSet('A', 'F'),
        Uri::CharacterSet('a', 'f')
    };

    /**
     * This is the character set corresponds to the "unreserved" syntax
     * specified in RFC 3986 (https://tools.ietf.org/html/rfc3986).
     */
    const Uri::CharacterSet UNRESERVED{
        ALPHA,
        DIGIT,
        '-', '.', '_', '~'
    };

    /**
     * This is the character set corresponds to the "sub-delims" syntax
     * specified in RFC 3986 (https://tools.ietf.org/html/rfc3986).
     */
    const Uri::CharacterSet SUB_DELIMS{
        '!', '$', '&', '\'', '(', ')',
        '*', '+', ',', ';', '='
    };

    /**
     * This is the character set corresponds to the second part
     * of the "scheme" syntax
     * specified in RFC 3986 (https://tools.ietf.org/html/rfc3986).
     */
    const Uri::CharacterSet SCHEME_NOT_FIRST{
        ALPHA,
        DIGIT,
        '+', '-', '.',
    };

    /**
     * This is the character set corresponds to the "pchar" syntax
     * specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
     * leaving out "pct-encoded".
     */
    const Uri::CharacterSet PCHAR_NOT_PCT_ENCODED{
        UNRESERVED,
        SUB_DELIMS,
        ':', '@'
    };

    /**
     * This is the character set corresponds to the "query" syntax
     * and the "fragment" syntax
     * specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
     * leaving out "pct-encoded".
     */
    const Uri::CharacterSet QUERY_OR_FRAGMENT_NOT_PCT_ENCODED{
        PCHAR_NOT_PCT_ENCODED,
        '/', '?'
    };

    /**
     * This is the character set corresponds to the "userinfo" syntax
     * specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
     * leaving out "pct-encoded".
     */
    const Uri::CharacterSet USER_INFO_NOT_PCT_ENCODED{
        UNRESERVED,
        SUB_DELIMS,
        ':',
    };

    /**
     * This is the character set corresponds to the "reg-name" syntax
     * specified in RFC 3986 (https://tools.ietf.org/html/rfc3986),
     * leaving out "pct-encoded".
     */
    const Uri::CharacterSet REG_NAME_NOT_PCT_ENCODED{
        UNRESERVED,
        SUB_DELIMS
    };

    /**
     * This is the character set corresponds to the last part of
     * the "IPvFuture" syntax
     * specified in RFC 3986 (https://tools.ietf.org/html/rfc3986).
     */
    const Uri::CharacterSet IPV_FUTURE_LAST_PART{
        UNRESERVED,
        SUB_DELIMS,
        ':'
    };

    /**
     * This function parses the given string as an unsigned 16-bit
     * integer, detecting invalid characters, overflow, etc.
     *
     * @param[in] numberString
     *     This is the string containing the number to parse.
     *
     * @param[out] number
     *     This is where to store the number parsed.
     *
     * @return
     *     An indication of whether or not the number was parsed
     *     successfully is returned.
     */
    bool ParseUint16(
        const std::string& numberString,
        uint16_t& number
    ) {
        uint32_t numberIn32Bits = 0;
        for (auto c: numberString) {
            if (
                (c < '0')
                || (c > '9')
            ) {
                return false;
            }
            numberIn32Bits *= 10;
            numberIn32Bits += (uint16_t)(c - '0');
            if (
                (numberIn32Bits & ~((1 << 16) - 1)) != 0
            ) {
                return false;
            }
        }
        number = (uint16_t)numberIn32Bits;
        return true;
    }

    /**
     * This function takes a given "stillPassing" strategy
     * and invokes it on the sequence of characters in the given
     * string, to check if the string passes or not.
     *
     * @param[in] candidate
     *     This is the string to test.
     *
     * @param[in] stillPassing
     *     This is the strategy to invoke in order to test the string.
     *
     * @return
     *     An indication of whether or not the given candidate string
     *     passes the test is returned.
     */
    bool FailsMatch(
        const std::string& candidate,
        std::function< bool(char, bool) > stillPassing
    ) {
        for (const auto c: candidate) {
            if (!stillPassing(c, false)) {
                return true;
            }
        }
        return !stillPassing(' ', true);
    }

    /**
     * This function returns a strategy function that
     * may be used with the FailsMatch function to test a scheme
     * to make sure it is legal according to the standard.
     *
     * @return
     *      A strategy function that may be used with the
     *      FailsMatch function to test a scheme to make sure
     *      it is legal according to the standard is returned.
     */
    std::function< bool(char, bool) > LegalSchemeCheckStrategy() {
        auto isFirstCharacter = std::make_shared< bool >(true);
        return [isFirstCharacter](char c, bool end){
            if (end) {
                return !*isFirstCharacter;
            } else {
                bool check;
                if (*isFirstCharacter) {
                    check = ALPHA.Contains(c);
                } else {
                    check = SCHEME_NOT_FIRST.Contains(c);
                }
                *isFirstCharacter = false;
                return check;
            }
        };
    }

    /**
     * This method checks and decodes the given URI element.
     * What we are calling a "URI element" is any part of the URI
     * which is a sequence of characters that:
     * - may be percent-encoded
     * - if not percent-encoded, are in a restricted set of characters
     *
     * @param[in,out] element
     *     On input, this is the element to check and decode.
     *     On output, this is the decoded element.
     *
     * @param[in] allowedCharacters
     *     This is the set of characters that do not need to
     *     be percent-encoded.
     *
     * @return
     *     An indication of whether or not the element
     *     passed all checks and was decoded successfully is returned.
     */
    bool DecodeElement(
        std::string& element,
        const Uri::CharacterSet& allowedCharacters
    ) {
        const auto originalSegment = std::move(element);
        element.clear();
        bool decodingPec = false;
        Uri::PercentEncodedCharacterDecoder pecDecoder;
        for (const auto c: originalSegment) {
            if (decodingPec) {
                if (!pecDecoder.NextEncodedCharacter(c)) {
                    return false;
                }
                if (pecDecoder.Done()) {
                    decodingPec = false;
                    element.push_back((char)pecDecoder.GetDecodedCharacter());
                }
            } else if (c == '%') {
                decodingPec = true;
                pecDecoder = Uri::PercentEncodedCharacterDecoder();
            } else {
                if (allowedCharacters.Contains(c)) {
                    element.push_back(c);
                } else {
                    return false;
                }
            }
        }
        return true;
    }

    /**
     * This method checks and decodes the given query or fragment.
     *
     * @param[in,out] queryOrFragment
     *     On input, this is the query or fragment to check and decode.
     *     On output, this is the decoded query or fragment.
     *
     * @return
     *     An indication of whether or not the query or fragment
     *     passed all checks and was decoded successfully is returned.
     */
    bool DecodeQueryOrFragment(std::string& queryOrFragment) {
        return DecodeElement(
            queryOrFragment,
            QUERY_OR_FRAGMENT_NOT_PCT_ENCODED
        );
    }

}

namespace Uri {
    /**
     * This contains the private properties of a Uri instance.
     */
    struct Uri::Impl {
        // Properties

        /**
         * This is the "scheme" element of the URI.
         */
        std::string scheme;

        /**
         * This is the "UserInfo" element of the URI.
         */
        std::string userInfo;

        /**
         * This is the "host" element of the URI.
         */
        std::string host;

        /**
         * This flag indicates whether or not the
         * URI includes a port number.
         */
        bool hasPort = false;

        /**
         * This is the port number element of the URI.
         */
        uint16_t port = 0;

        /**
         * This is the "path" element of the URI,
         * as a sequence of segments.
         */
        std::vector< std::string > path;

        /**
         * This is the "query" element of the URI,
         * if it has one.
         */
        std::string query;

        /**
         * This is the "fragment" element of the URI,
         * if it has one.
         */
        std::string fragment;

        // Methods

        /**
         * This method builds the internal path element sequence
         * by parsing it from the given path string.
         *
         * @param[in] pathString
         *     This is the string containing the whole path of the URI.
         *
         * @return
         *     An indication if the path was parsed correctly or not
         *     is returned.
         */
        bool ParsePath(std::string pathString) {
            path.clear();
            if (pathString == "/") {
                // Special case of a path that is empty but needs a single
                // empty-string element to indicate that it is absolute.
                path.push_back("");
                pathString.clear();
            } else if (!pathString.empty()) {
                for(;;) {
                    auto pathDelimiter = pathString.find('/');
                    if (pathDelimiter == std::string::npos) {
                        path.push_back(pathString);
                        pathString.clear();
                        break;
                    } else {
                        path.emplace_back(
                            pathString.begin(),
                            pathString.begin() + pathDelimiter
                        );
                        pathString = pathString.substr(pathDelimiter + 1);
                    }
                }
            }
            for (auto& segment: path) {
                if (!DecodeElement(segment, PCHAR_NOT_PCT_ENCODED)) {
                    return false;
                }
            }
            return true;
        }

        /**
         * This method parses the elements that make up the authority
         * composite part of the URI,  by parsing it from the given string.
         *
         * @param[in] authorityString
         *     This is the string containing the whole authority part
         *     of the URI.
         *
         * @return
         *     An indication if the path was parsed correctly or not
         *     is returned.
         */
        bool ParseAuthority(const std::string& authorityString) {
            /**
             * These are the various states for the state machine implemented
             * below to correctly split up and validate the URI substring
             * containing the host and potentially a port number as well.
             */
            enum class HostParsingState {
                FIRST_CHARACTER,
                NOT_IP_LITERAL,
                PERCENT_ENCODED_CHARACTER,
                IP_LITERAL,
                IPV6_ADDRESS,
                IPV_FUTURE_NUMBER,
                IPV_FUTURE_BODY,
                GARBAGE_CHECK,
                PORT,
            };

            // Next, check if there is a UserInfo, and if so, extract it.
            const auto userInfoDelimiter = authorityString.find('@');
            std::string hostPortString;
            userInfo.clear();
            if (userInfoDelimiter == std::string::npos) {
                hostPortString = authorityString;
            } else {
                userInfo = authorityString.substr(0, userInfoDelimiter);
                if (!DecodeElement(userInfo, USER_INFO_NOT_PCT_ENCODED)) {
                    return false;
                }
                hostPortString = authorityString.substr(userInfoDelimiter + 1);
            }

            // Next, parsing host and port from authority and path.
            std::string portString;
            HostParsingState hostParsingState = HostParsingState::FIRST_CHARACTER;
            int decodedCharacter = 0;
            host.clear();
            PercentEncodedCharacterDecoder pecDecoder;
            bool hostIsRegName = false;
            for (const auto c: hostPortString) {
                switch(hostParsingState) {
                    case HostParsingState::FIRST_CHARACTER: {
                        if (c == '[') {
                            host.push_back(c);
                            hostParsingState = HostParsingState::IP_LITERAL;
                            break;
                        } else {
                            hostParsingState = HostParsingState::NOT_IP_LITERAL;
                            hostIsRegName = true;
                        }
                    }

                    case HostParsingState::NOT_IP_LITERAL: {
                        if (c == '%') {
                            pecDecoder = PercentEncodedCharacterDecoder();
                            hostParsingState = HostParsingState::PERCENT_ENCODED_CHARACTER;
                        } else if (c == ':') {
                            hostParsingState = HostParsingState::PORT;
                        } else {
                            if (REG_NAME_NOT_PCT_ENCODED.Contains(c)) {
                                host.push_back(c);
                            } else {
                                return false;
                            }
                        }
                    } break;

                    case HostParsingState::PERCENT_ENCODED_CHARACTER: {
                        if (!pecDecoder.NextEncodedCharacter(c)) {
                            return false;
                        }
                        if (pecDecoder.Done()) {
                            hostParsingState = HostParsingState::NOT_IP_LITERAL;
                            host.push_back((char)pecDecoder.GetDecodedCharacter());
                        }
                    } break;

                    case HostParsingState::IP_LITERAL: {
                        if (c == 'v') {
                            host.push_back(c);
                            hostParsingState = HostParsingState::IPV_FUTURE_NUMBER;
                            break;
                        } else {
                            hostParsingState = HostParsingState::IPV6_ADDRESS;
                        }
                    }

                    case HostParsingState::IPV6_ADDRESS: {
                        // TODO: research this offline first
                        // before attempting to code it
                        host.push_back(c);
                        if (c == ']') {
                            hostParsingState = HostParsingState::GARBAGE_CHECK;
                        }
                    } break;

                    case HostParsingState::IPV_FUTURE_NUMBER: {
                        if (c == '.') {
                            hostParsingState = HostParsingState::IPV_FUTURE_BODY;
                        } else if (!HEXDIG.Contains(c)) {
                            return false;
                        }
                        host.push_back(c);
                    } break;

                    case HostParsingState::IPV_FUTURE_BODY: {
                        host.push_back(c);
                        if (c == ']') {
                            hostParsingState = HostParsingState::GARBAGE_CHECK;
                        } else if (!IPV_FUTURE_LAST_PART.Contains(c)) {
                            return false;
                        }
                    } break;

                    case HostParsingState::GARBAGE_CHECK: {
                        // illegal to have anything else, unless it's a colon,
                        // in which case it's a port delimiter
                        if (c == ':') {
                            hostParsingState = HostParsingState::PORT;
                        } else {
                            return false;
                        }
                    } break;

                    case HostParsingState::PORT: {
                        portString.push_back(c);
                    } break;
                }
            }
            if (hostIsRegName) {
                host = NormalizeCaseInsensitiveString(host);
            }
            if (portString.empty()) {
                hasPort = false;
            } else {
                if (!ParseUint16(portString, port)) {
                    return false;
                }
                hasPort = true;
            }
            return true;
        }

        /**
         * This method takes an unparsed URI string and separates out
         * the scheme (if any) and parses it, returning the remainder
         * of the unparsed URI string.
         *
         * @param[in] authorityAndPathString
         *     This is the the part of an unparsed URI consisting
         *     of the authority (if any) followed by the path.
         *
         * @param[out] pathString
         *     This is where to store the the path
         *     part of the input string.
         *
         * @return
         *     An indication of whether or not the given input string
         *     was successfully parsed is returned.
         */
        bool ParseScheme(
            const std::string& uriString,
            std::string& rest
        ) {
            // Limit our search so we don't scan into the authority
            // or path elements, because these may have the colon
            // character as well, which we might misinterpret
            // as the scheme delimiter.
            auto authorityOrPathDelimiterStart = uriString.find('/');
            if (authorityOrPathDelimiterStart == std::string::npos) {
                authorityOrPathDelimiterStart = uriString.length();
            }
            const auto schemeEnd = uriString.substr(0, authorityOrPathDelimiterStart).find(':');
            if (schemeEnd == std::string::npos) {
                scheme.clear();
                rest = uriString;
            } else {
                scheme = uriString.substr(0, schemeEnd);
                bool isFirstCharacter = true;
                if (
                    FailsMatch(
                        scheme,
                        LegalSchemeCheckStrategy()
                    )
                ) {
                    return false;
                }
                scheme = NormalizeCaseInsensitiveString(scheme);
                rest = uriString.substr(schemeEnd + 1);
            }
        }

        /**
         * This method takes the part of an unparsed URI consisting
         * of the authority (if any) followed by the path, and divides
         * it into the authority and path parts, storing any authority
         * information in the internal state, and returning the path
         * part of the input string.
         *
         * @param[in] authorityAndPathString
         *     This is the the part of an unparsed URI consisting
         *     of the authority (if any) followed by the path.
         *
         * @param[out] pathString
         *     This is where to store the the path
         *     part of the input string.
         *
         * @return
         *     An indication of whether or not the given input string
         *     was successfully parsed is returned.
         */
        bool SplitAuthorityFromPathAndParseIt(
            std::string authorityAndPathString,
            std::string& pathString
        ) {
            // Split authority from path.  If there is an authority, parse it.
            if (authorityAndPathString.substr(0, 2) == "//") {
                // Strip off authority marker.
                authorityAndPathString = authorityAndPathString.substr(2);

                // First separate the authority from the path.
                auto authorityEnd = authorityAndPathString.find('/');
                if (authorityEnd == std::string::npos) {
                    authorityEnd = authorityAndPathString.length();
                }
                pathString = authorityAndPathString.substr(authorityEnd);
                auto authorityString = authorityAndPathString.substr(0, authorityEnd);

                // Parse the elements inside the authority string.
                if (!ParseAuthority(authorityString)) {
                    return false;
                }
            } else {
                userInfo.clear();
                host.clear();
                hasPort = false;
                pathString = authorityAndPathString;
            }
        }

        /**
         * This method handles the special case of the URI having an
         * authority but having an empty path.  In this case it sets
         * the path as "/".
         */
        void SetDefaultPathIfAuthorityPresentAndPathEmpty() {
            if (
                !host.empty()
                && path.empty()
            ) {
                path.push_back("");
            }
        }

        /**
         * This method takes the part of a URI string that has just
         * the query element with its delimiter, and breaks off
         * and decodes the query.
         *
         * @param[in] queryWithDelimiter
         *     This is the part of a URI string that has just
         *     the query element with its delimiter.
         *
         * @return
         *     An indication of whether or not the method succeeded
         *     is returned.
         */
        bool ParseQuery(const std::string& queryWithDelimiter) {
            if (queryWithDelimiter.empty()) {
                query.clear();
            } else {
                query = queryWithDelimiter.substr(1);
            }
            return DecodeQueryOrFragment(query);
        }

        /**
         * This method takes the part of a URI string that has just
         * the query and/or fragment elements, and breaks off
         * and decodes the fragment part, returning the rest,
         * which will be either empty or have the query with the
         * query delimiter still attached.
         *
         * @param[in] queryAndOrFragment
         *     This is the part of a URI string that has just
         *     the query and/or fragment elements.
         *
         * @param[out] rest
         *     This is where to store the rest of the input string
         *     after removing any fragment and fragment delimiter.
         *
         * @return
         *     An indication of whether or not the method succeeded
         *     is returned.
         */
        bool ParseFragment(
            const std::string& queryAndOrFragment,
            std::string& rest
        ) {
            const auto fragmentDelimiter = queryAndOrFragment.find('#');
            if (fragmentDelimiter == std::string::npos) {
                fragment.clear();
                rest = queryAndOrFragment;
            } else {
                fragment = queryAndOrFragment.substr(fragmentDelimiter + 1);
                rest = queryAndOrFragment.substr(0, fragmentDelimiter);
            }
            return DecodeQueryOrFragment(fragment);
        }

        /**
         * This method applies the "remove_dot_segments" routine talked about
         * in RFC 3986 (https://tools.ietf.org/html/rfc3986) to the path
         * segments of the URI, in order to normalize the path
         * (apply and remove "." and ".." segments).
         */
        void NormalizePath() {
            auto oldPath = std::move(path);
            path.clear();
            bool isAbsolute = (
                !oldPath.empty()
                && oldPath[0].empty()
            );
            bool atDirectoryLevel = false;
            for (const auto segment: oldPath) {
                if (segment == ".") {
                    atDirectoryLevel = true;
                } else if (segment == "..") {
                    if (!path.empty()) {
                        if (
                            !isAbsolute
                            || (path.size() > 1)
                        ) {
                            path.pop_back();
                        }
                    }
                    atDirectoryLevel = true;
                } else {
                    if (
                        !atDirectoryLevel
                        || !segment.empty()
                    ) {
                        path.push_back(segment);
                    }
                    atDirectoryLevel = segment.empty();
                }
            }
            if (
                atDirectoryLevel
                && (
                    !path.empty()
                    && !path.back().empty()
                )
            ) {
                path.push_back("");
            }
        }

        /**
         * This method replaces the URI's scheme with that of
         * another URI.
         *
         * @param[in] other
         *     This is the other URI from which to copy the scheme.
         */
        void CopyScheme(const Uri& other) {
            scheme = other.impl_->scheme;
        }

        /**
         * This method replaces the URI's authority with that of
         * another URI.
         *
         * @param[in] other
         *     This is the other URI from which to copy the authority.
         */
        void CopyAuthority(const Uri& other) {
            host = other.impl_->host;
            userInfo = other.impl_->userInfo;
            hasPort = other.impl_->hasPort;
            port = other.impl_->port;
        }

        /**
         * This method replaces the URI's path with that of
         * another URI.
         *
         * @param[in] other
         *     This is the other URI from which to copy the path.
         */
        void CopyPath(const Uri& other) {
            path = other.impl_->path;
        }

        /**
         * This method replaces the URI's path with that of
         * the normalized form of another URI.
         *
         * @param[in] other
         *     This is the other URI from which to copy
         *     the normalized path.
         */
        void CopyAndNormalizePath(const Uri& other) {
            CopyPath(other);
            NormalizePath();
        }

        /**
         * This method replaces the URI's query with that of
         * another URI.
         *
         * @param[in] other
         *     This is the other URI from which to copy the query.
         */
        void CopyQuery(const Uri& other) {
            query = other.impl_->query;
        }

        /**
         * This method replaces the URI's fragment with that of
         * another URI.
         *
         * @param[in] other
         *     This is the other URI from which to copy the query.
         */
        void CopyFragment(const Uri& other) {
            fragment = other.impl_->fragment;
        }

        /**
         * This method returns an indication of whether or not the
         * path of the URI is an absolute path, meaning it begins
         * with a forward slash ('/') character.
         *
         * @return
         *     An indication of whether or not the path of the URI
         *     is an absolute path, meaning it begins
         *     with a forward slash ('/') character is returned.
         */
        bool IsPathAbsolute() const {
            return (
                !path.empty()
                && (path[0] == "")
            );
        }
    };

    Uri::~Uri() = default;
    Uri::Uri(Uri&&) = default;
    Uri& Uri::operator=(Uri&&) = default;

    Uri::Uri()
        : impl_(new Impl)
    {
    }

    bool Uri::operator==(const Uri& other) const {
        return (
            (impl_->scheme == other.impl_->scheme)
            && (impl_->userInfo == other.impl_->userInfo)
            && (impl_->host == other.impl_->host)
            && (
                (!impl_->hasPort && !other.impl_->hasPort)
                || (
                    (impl_->hasPort && other.impl_->hasPort)
                    && (impl_->port == other.impl_->port)
                )
            )
            && (impl_->path == other.impl_->path)
            && (impl_->query == other.impl_->query)
            && (impl_->fragment == other.impl_->fragment)
        );
    }

    bool Uri::operator!=(const Uri& other) const {
        return !(*this == other);
    }

    bool Uri::ParseFromString(const std::string& uriString) {
        std::string rest;
        if (!impl_->ParseScheme(uriString, rest)) {
            return false;
        }
        const auto pathEnd = rest.find_first_of("?#");
        const auto authorityAndPathString = rest.substr(0, pathEnd);
        const auto queryAndOrFragment = rest.substr(authorityAndPathString.length());
        std::string pathString;
        if (!impl_->SplitAuthorityFromPathAndParseIt(authorityAndPathString, pathString)) {
            return false;
        }
        if (!impl_->ParsePath(pathString)) {
            return false;
        }
        impl_->SetDefaultPathIfAuthorityPresentAndPathEmpty();
        if (!impl_->ParseFragment(queryAndOrFragment, rest)) {
            return false;
        }
        return impl_->ParseQuery(rest);
    }

    std::string Uri::GetScheme() const {
        return impl_->scheme;
    }

    std::string Uri::GetUserInfo() const {
        return impl_->userInfo;
    }

    std::string Uri::GetHost() const {
        return impl_->host;
    }

    std::vector< std::string > Uri::GetPath() const {
        return impl_->path;
    }

    bool Uri::HasPort() const {
        return impl_->hasPort;
    }

    uint16_t Uri::GetPort() const {
        return impl_->port;
    }

    bool Uri::IsRelativeReference() const {
        return impl_->scheme.empty();
    }

    bool Uri::ContainsRelativePath() const {
        return !impl_->IsPathAbsolute();
    }

    std::string Uri::GetQuery() const {
        return impl_->query;
    }

    std::string Uri::GetFragment() const {
        return impl_->fragment;
    }

    void Uri::NormalizePath() {
        impl_->NormalizePath();
    }

    Uri Uri::Resolve(const Uri& relativeReference) const {
        // Resolve the reference by following the algorithm
        // from section 5.2.2 in
        // RFC 3986 (https://tools.ietf.org/html/rfc3986).
        Uri target;
        if (!relativeReference.impl_->scheme.empty()) {
            target.impl_->CopyScheme(relativeReference);
            target.impl_->CopyAuthority(relativeReference);
            target.impl_->CopyAndNormalizePath(relativeReference);
            target.impl_->CopyQuery(relativeReference);
        } else {
            if (!relativeReference.impl_->host.empty()) {
                target.impl_->CopyAuthority(relativeReference);
                target.impl_->CopyAndNormalizePath(relativeReference);
                target.impl_->CopyQuery(relativeReference);
            } else {
                if (relativeReference.impl_->path.empty()) {
                    target.impl_->path = impl_->path;
                    if (!relativeReference.impl_->query.empty()) {
                        target.impl_->CopyQuery(relativeReference);
                    } else {
                        target.impl_->CopyQuery(*this);
                    }
                } else {
                    // RFC describes this as:
                    // "if (R.path starts-with "/") then"
                    if (relativeReference.impl_->IsPathAbsolute()) {
                        target.impl_->CopyAndNormalizePath(relativeReference);
                    } else {
                        // RFC describes this as:
                        // "T.path = merge(Base.path, R.path);"
                        target.impl_->CopyPath(*this);
                        if (target.impl_->path.size() > 1) {
                            target.impl_->path.pop_back();
                        }
                        std::copy(
                            relativeReference.impl_->path.begin(),
                            relativeReference.impl_->path.end(),
                            std::back_inserter(target.impl_->path)
                        );
                        target.NormalizePath();
                    }
                    target.impl_->CopyQuery(relativeReference);
                }
                target.impl_->CopyAuthority(*this);
            }
            target.impl_->CopyScheme(*this);
        }
        target.impl_->CopyFragment(relativeReference);
        return target;
    }
}
