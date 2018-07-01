/**
 * @file Uri.cpp
 *
 * This module contains the implementation of the Uri::Uri class.
 *
 * © 2018 by Richard Walters
 */

#include <functional>
#include <inttypes.h>
#include <memory>
#include <string>
#include <Uri/Uri.hpp>
#include <vector>

namespace {

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

    bool IsCharacterInSet(
        char c,
        std::initializer_list< char > characterSet
    ) {
        for (
            auto charInSet = characterSet.begin();
            charInSet != characterSet.end();
            ++charInSet
        ) {
            const auto first = *charInSet++;
            const auto last = *charInSet;
            if (
                (c >= first)
                && (c <= last)
            ) {
                return true;
            }
        }
        return false;
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
                    check = IsCharacterInSet(c, { 'a','z', 'A','Z' });
                } else {
                    check = IsCharacterInSet(c, { 'a','z', 'A','Z', '0','9', '+','+', '-','-', '.','.' });
                }
                *isFirstCharacter = false;
                return check;
            }
        };
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
            // Next, check if there is a UserInfo, and if so, extract it.
            const auto userInfoDelimiter = authorityString.find('@');
            std::string hostPortString;
            userInfo.clear();
            if (userInfoDelimiter == std::string::npos) {
                hostPortString = authorityString;
            } else {
                const auto userInfoEncoded = authorityString.substr(0, userInfoDelimiter);
                size_t decoderState = 0;
                int decodedCharacter = 0;
                for (const auto c: userInfoEncoded) {
                    switch(decoderState) {
                        case 0: { // default
                            if (c == '%') {
                                decoderState = 1;
                            } else {
                                if (
                                    IsCharacterInSet(
                                        c,
                                        {
                                            // unreserved
                                            'a','z', 'A','Z', // ALPHA
                                            '0','9', // DIGIT
                                            '-','-', '.','.', '_','_', '~','~',

                                            // sub-delims
                                            '!','!', '$','$', '&','&', '\'','\'', '(','(', ')',')',
                                            '*','*', '+','+', ',',',', ';',';', '=','=',

                                            // (also allowed in userinfo)
                                            ':',':',
                                        }
                                    )
                                ) {
                                    userInfo.push_back(c);
                                } else {
                                    return false;
                                }
                            }
                        } break;

                        case 1: { // % ...
                            decoderState = 2;
                            decodedCharacter <<= 4;
                            if (IsCharacterInSet(c, {'0','9'})) {
                                decodedCharacter += (int)(c - '0');
                            } else if (IsCharacterInSet(c, {'A','F'})) {
                                decodedCharacter += (int)(c - 'A') + 10;
                            } else {
                                return false;
                            }
                        } break;

                        case 2: { // %[0-9A-F] ...
                            decoderState = 0;
                            decodedCharacter <<= 4;
                            if (IsCharacterInSet(c, {'0','9'})) {
                                decodedCharacter += (int)(c - '0');
                            } else if (IsCharacterInSet(c, {'A','F'})) {
                                decodedCharacter += (int)(c - 'A') + 10;
                            } else {
                                return false;
                            }
                            userInfo.push_back((char)decodedCharacter);
                        } break;
                    }
                }
                hostPortString = authorityString.substr(userInfoDelimiter + 1);
            }

            // Next, parsing host and port from authority and path.
            const auto portDelimiter = hostPortString.find(':');
            if (portDelimiter == std::string::npos) {
                host = hostPortString;
                hasPort = false;
            } else {
                host = hostPortString.substr(0, portDelimiter);
                const auto portString = hostPortString.substr(portDelimiter + 1);
                if (!ParseUint16(portString, port)) {
                    return false;
                }
                hasPort = true;
            }
            return true;
        }
    };

    Uri::~Uri() = default;

    Uri::Uri()
        : impl_(new Impl)
    {
    }

    bool Uri::ParseFromString(const std::string& uriString) {
        // First parse the scheme.
        const auto schemeEnd = uriString.find(':');
        std::string rest;
        if (schemeEnd == std::string::npos) {
            impl_->scheme.clear();
            rest = uriString;
        } else {
            impl_->scheme = uriString.substr(0, schemeEnd);
            bool isFirstCharacter = true;
            if (
                FailsMatch(
                    impl_->scheme,
                    LegalSchemeCheckStrategy()
                )
            ) {
                return false;
            }
            rest = uriString.substr(schemeEnd + 1);
        }

        // Next parse the authority.
        const auto pathEnd = rest.find_first_of("?#");
        auto authorityAndPathString = rest.substr(0, pathEnd);
        const auto queryAndOrFragment = rest.substr(authorityAndPathString.length());
        std::string pathString;
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
            if (!impl_->ParseAuthority(authorityString)) {
                return false;
            }
        } else {
            impl_->userInfo.clear();
            impl_->host.clear();
            impl_->hasPort = false;
            pathString = authorityAndPathString;
        }

        // Next, parse the path.
        if (!impl_->ParsePath(pathString)) {
            return false;
        }

        // Next, parse the fragment if there is one.
        const auto fragmentDelimiter = queryAndOrFragment.find('#');
        if (fragmentDelimiter == std::string::npos) {
            impl_->fragment.clear();
            rest = queryAndOrFragment;
        } else {
            impl_->fragment = queryAndOrFragment.substr(fragmentDelimiter + 1);
            rest = queryAndOrFragment.substr(0, fragmentDelimiter);
        }

        // Finally, if anything is left, it's the query.
        if (rest.empty()) {
            impl_->query.clear();
        } else {
            impl_->query = rest.substr(1);
        }
        return true;
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
        if (impl_->path.empty()) {
            return true;
        } else {
            return !impl_->path[0].empty();
        }
    }

    std::string Uri::GetQuery() const {
        return impl_->query;
    }

    std::string Uri::GetFragment() const {
        return impl_->fragment;
    }

}
