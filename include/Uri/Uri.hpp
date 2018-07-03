#ifndef URI_HPP
#define URI_HPP

/**
 * @file Uri.hpp
 *
 * This module declares the Uri::Uri class.
 *
 * © 2018 by Richard Walters
 */

#include <memory>
#include <stdint.h>
#include <string>
#include <vector>

namespace Uri {

    /**
     * This class represents a Uniform Resource Identifier (URI),
     * as defined in RFC 3986 (https://tools.ietf.org/html/rfc3986).
     */
    class Uri {
        // Lifecycle management
    public:
        ~Uri();
        Uri(const Uri&) = delete;
        Uri(Uri&&) = delete;
        Uri& operator=(const Uri&) = delete;
        Uri& operator=(Uri&&) = delete;

        // Public methods
    public:
        /**
         * This is the default constructor.
         */
        Uri();

        /**
         * This method builds the URI from the elements parsed
         * from the given string rendering of a URI.
         *
         * @param[in] uriString
         *     This is the string rendering of the URI to parse.
         *
         * @return
         *     An indication of whether or not the URI was
         *     parsed successfully is returned.
         */
        bool ParseFromString(const std::string& uriString);

        /**
         * This method returns the "scheme" element of the URI.
         *
         * @return
         *     The "scheme" element of the URI is returned.
         *
         * @retval ""
         *     This is returned if there is no "scheme" element in the URI.
         */
        std::string GetScheme() const;

        /**
         * This method returns the "UserInfo" element of the URI.
         *
         * @return
         *     The "UserInfo" element of the URI is returned.
         *
         * @retval ""
         *     This is returned if there is no "UserInfo" element in the URI.
         */
        std::string GetUserInfo() const;

        /**
        * This method returns the "host" element of the URI.
        *
        * @return
        *     The "host" element of the URI is returned.
        *
        * @retval ""
        *     This is returned if there is no "host" element in the URI.
        */
        std::string GetHost() const;

        /**
         * This method returns the "path" element of the URI,
         * as a sequence of segments.
         *
         * @note
         *     If the first segment of the path is an empty string,
         *     then the URI has an absolute path.
         *
         * @return
         *     The "path" element of the URI is returned
         *     as a sequence of segments.
         */
        std::vector< std::string > GetPath() const;

        /**
         * This method returns an indication of whether or not the
         * URI includes a port number.
         *
         * @return
         *     An indication of whether or not the
         *     URI includes a port number is returned.
         */
        bool HasPort() const;

        /**
         * This method returns the port number element of the URI,
         * if it has one.
         *
         * @return
         *     The port number element of the URI is returned.
         *
         * @note
         *     The returned port number is only valid if the
         *     HasPort method returns true.
         */
        uint16_t GetPort() const;

        /**
         * This method returns an indication of whether or not
         * the URI is a relative reference.
         *
         * @return
         *     An indication of whether or not the URI is a
         *     relative reference is returned.
         */
        bool IsRelativeReference() const;

        /**
         * This method returns an indication of whether or not
         * the URI contains a relative path.
         *
         * @return
         *     An indication of whether or not the URI contains a
         *     relative path is returned.
         */
        bool ContainsRelativePath() const;

        /**
         * This method returns the "query" element of the URI,
         * if it has one.
         *
         * @return
         *     The "query" element of the URI is returned.
         *
         * @retval ""
         *     This is returned if there is no "query" element in the URI.
         */
        std::string GetQuery() const;

        /**
         * This method returns the "fragment" element of the URI,
         * if it has one.
         *
         * @return
         *     The "fragment" element of the URI is returned.
         *
         * @retval ""
         *     This is returned if there is no "fragment" element in the URI.
         */
        std::string GetFragment() const;

        /**
         * This method applies the "remove_dot_segments" routine talked about
         * in RFC 3986 (https://tools.ietf.org/html/rfc3986) to the path
         * segments of the URI, in order to normalize the path
         * (apply and remove "." and ".." segments).
         */
        void NormalizePath();

        // Private properties
    private:
        /**
         * This is the type of structure that contains the private
         * properties of the instance.  It is defined in the implementation
         * and declared here to ensure that it is scoped inside the class.
         */
        struct Impl;

        /**
         * This contains the private properties of the instance.
         */
        std::unique_ptr< struct Impl > impl_;
    };

}

#endif /* URI_HPP */
