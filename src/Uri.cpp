/**
 * @file Uri.cpp
 *
 * This module contains the implementation of the Uri::Uri class.
 *
 * © 2018 by Richard Walters
 */

#include <inttypes.h>
#include <string>
#include <Uri/Uri.hpp>
#include <vector>

namespace Uri {
    /**
     * This contains the private properties of a Uri instance.
     */
    struct Uri::Impl {
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
            rest = uriString.substr(schemeEnd + 1);
        }

        // Next parse the authority.
        impl_->hasPort = false;
        const auto pathEnd = rest.find_first_of("?#");
        auto authorityAndPathString = rest.substr(0, pathEnd);
        const auto queryAndOrFragment = rest.substr(authorityAndPathString.length());
        std::string hostPortAndPathString;
        if (authorityAndPathString.substr(0, 2) == "//") {
            // Strip off authority marker.
            authorityAndPathString = authorityAndPathString.substr(2);

            // First separate the authority from the path.
            auto authorityEnd = authorityAndPathString.find('/');
            if (authorityEnd == std::string::npos) {
                authorityEnd = authorityAndPathString.length();
            }

            // Next, check if there is a UserInfo, and if so, extract it.
            const auto userInfoDelimiter = authorityAndPathString.find('@');
            if (userInfoDelimiter == std::string::npos) {
                impl_->userInfo.clear();
                hostPortAndPathString = authorityAndPathString;
            } else {
                impl_->userInfo = authorityAndPathString.substr(0, userInfoDelimiter);
                hostPortAndPathString = authorityAndPathString.substr(userInfoDelimiter + 1);
            }

            // Next, parsing host and port from authority and path.
            const auto portDelimiter = hostPortAndPathString.find(':');
            if (portDelimiter == std::string::npos) {
                impl_->host = hostPortAndPathString.substr(0, authorityEnd);
            } else {
                impl_->host = hostPortAndPathString.substr(0, portDelimiter);
                uint32_t newPort = 0;
                for (auto c: hostPortAndPathString.substr(portDelimiter + 1, authorityEnd - portDelimiter - 1)) {
                    if (
                        (c < '0')
                        || (c > '9')
                    ) {
                        return false;
                    }
                    newPort *= 10;
                    newPort += (uint16_t)(c - '0');
                    if (
                        (newPort & ~((1 << 16) - 1)) != 0
                    ) {
                        return false;
                    }
                }
                impl_->port = (uint16_t)newPort;
                impl_->hasPort = true;
            }
            hostPortAndPathString = authorityAndPathString.substr(authorityEnd);
        } else {
            impl_->host.clear();
            hostPortAndPathString = authorityAndPathString;
        }
        auto pathString = hostPortAndPathString;

        // Next, parse the path.
        impl_->path.clear();
        if (pathString == "/") {
            // Special case of a path that is empty but needs a single
            // empty-string element to indicate that it is absolute.
            impl_->path.push_back("");
            pathString.clear();
        } else if (!pathString.empty()) {
            for(;;) {
                auto pathDelimiter = pathString.find('/');
                if (pathDelimiter == std::string::npos) {
                    impl_->path.push_back(pathString);
                    pathString.clear();
                    break;
                } else {
                    impl_->path.emplace_back(
                        pathString.begin(),
                        pathString.begin() + pathDelimiter
                    );
                    pathString = pathString.substr(pathDelimiter + 1);
                }
            }
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
