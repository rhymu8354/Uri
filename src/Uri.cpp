/**
 * @file Uri.cpp
 *
 * This module contains the implementation of the Uri::Uri class.
 *
 * © 2018 by Richard Walters
 */

#include <string>
#include <Uri/Uri.hpp>
#include <vector>

namespace Uri {
    /**
     * This contains the private properties of a Uri instance.
     */
    struct Uri::Impl {
        /**
         * This is the character or character sequence
         * that should be interpreted as a path delimiter.
         */
        std::string pathDelimiter = "/";

        /**
         * This is the "scheme" element of the URI.
         */
        std::string scheme;

        /**
         * This is the "host" element of the URI.
         */
        std::string host;

        /**
         * This is the "path" element of the URI,
         * as a sequence of steps.
         */
        std::vector< std::string > path;
    };

    Uri::~Uri() = default;

    Uri::Uri()
        : impl_(new Impl)
    {
    }

    void Uri::SetPathDelimiter(const std::string& newPathDelimiter) {
        impl_->pathDelimiter = newPathDelimiter;
    }

    bool Uri::ParseFromString(const std::string& uriString) {
        // First parse the scheme.
        const auto schemeEnd = uriString.find(':');
        impl_->scheme = uriString.substr(0, schemeEnd);
        auto rest = uriString.substr(schemeEnd + 1);

        // Next parse the host.
        if (rest.substr(0, 2) == "//") {
            const auto authorityEnd = rest.find(impl_->pathDelimiter, 2);
            impl_->host = rest.substr(2, authorityEnd - 2);
            rest = rest.substr(authorityEnd);
        } else {
            impl_->host.clear();
        }

        // Finally, parse the path.
        impl_->path.clear();
        if (rest == impl_->pathDelimiter) {
            // Special case of a path that is empty but needs a single
            // empty-string element to indicate that it is absolute.
            impl_->path.push_back("");
        } else if (!rest.empty()) {
            for(;;) {
                auto pathDelimiter = rest.find(impl_->pathDelimiter);
                if (pathDelimiter == std::string::npos) {
                    impl_->path.push_back(rest);
                    break;
                } else {
                    impl_->path.emplace_back(
                        rest.begin(),
                        rest.begin() + pathDelimiter
                    );
                    rest = rest.substr(pathDelimiter + impl_->pathDelimiter.length());
                }
            }
        }
        return true;
    }

    std::string Uri::GetScheme() const {
        return impl_->scheme;
    }

    std::string Uri::GetHost() const {
        return impl_->host;
    }

    std::vector< std::string > Uri::GetPath() const {
        return impl_->path;
    }

}
