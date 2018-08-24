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
        ~Uri() noexcept;
        Uri(const Uri& other);
        Uri(Uri&&) noexcept;
        Uri& operator=(const Uri& other);
        Uri& operator=(Uri&&) noexcept;

        // Public methods
    public:
        /**
         * This is the default constructor.
         */
        Uri();

        /**
         * This is the equality comparison operator for the class.
         *
         * @param[in] other
         *     This is the other URI to which to compare this URI.
         *
         * @return
         *     An indication of whether or not the two URIs are
         *     equal is returned.
         */
        bool operator==(const Uri& other) const;

        /**
         * This is the inequality comparison operator for the class.
         *
         * @param[in] other
         *     This is the other URI to which to compare this URI.
         *
         * @return
         *     An indication of whether or not the two URIs are
         *     not equal is returned.
         */
        bool operator!=(const Uri& other) const;

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
         * This method returns an indication of whether or not the
         * URI includes a query.
         *
         * @return
         *     An indication of whether or not the
         *     URI includes a query is returned.
         */
        bool HasQuery() const;

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
         * This method returns an indication of whether or not the
         * URI includes a fragment.
         *
         * @return
         *     An indication of whether or not the
         *     URI includes a fragment is returned.
         */
        bool HasFragment() const;

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

        /**
         * This method resolves the given relative reference, based on the given
         * base URI, returning the resolved target URI.
         *
         * @param[in] relativeReference
         *     This describes how to get to the target starting at the base.
         *
         * @return
         *     The resolved target URI is returned.
         *
         * @note
         *     It only makes sense to call this method on an absolute URI
         *     (in which I mean, the base URI should be absolute,
         *     as in IsRelativeReference() should return false).
         */
        Uri Resolve(const Uri& relativeReference) const;

        /**
         * This method sets the scheme element of the URI.
         *
         * @param[in] scheme
         *     This is the scheme to set for the URI.
         */
        void SetScheme(const std::string& scheme);

        /**
         * This method sets the userinfo element of the URI.
         *
         * @param[in] userinfo
         *     This is the userinfo to set for the URI.
         */
        void SetUserInfo(const std::string& userinfo);

        /**
         * This method sets the host element of the URI.
         *
         * @param[in] host
         *     This is the host to set for the URI.
         */
        void SetHost(const std::string& host);

        /**
         * This method sets the port element of the URI.
         *
         * @param[in] port
         *     This is the port to set for the URI.
         */
        void SetPort(uint16_t port);

        /**
         * This method removes the port element from the URI.
         */
        void ClearPort();

        /**
         * This method sets the path element of the URI.
         *
         * @param[in] path
         *     This is the sequence of segments to use to form the path
         *     to set for the URI.
         *
         *     An empty string segment can be used at the front to
         *     indicate an absolute path (as opposed to a relative one).
         *
         *     An empty string segment can be used at the back to
         *     make sure the path ends in a delimiter (forward slash)
         *     when printed out or when combined with another URI
         *     via the Resolve() method.
         */
        void SetPath(const std::vector< std::string >& path);

        /**
         * This method removes the query element from the URI.
         */
        void ClearQuery();

        /**
         * This method sets the query element of the URI.
         *
         * @param[in] query
         *     This is the query to set for the URI.
         */
        void SetQuery(const std::string& query);

        /**
         * This method removes the fragment element from the URI.
         */
        void ClearFragment();

        /**
         * This method sets the fragment element of the URI.
         *
         * @param[in] fragment
         *     This is the fragment to set for the URI.
         */
        void SetFragment(const std::string& fragment);

        /**
         * This method constructs and returns the string
         * rendering of the URI, according to the rules
         * in RFC 3986 (https://tools.ietf.org/html/rfc3986).
         *
         * @return
         *     The string rendering of the URI is returned.
         */
        std::string GenerateString() const;

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
        std::unique_ptr< Impl > impl_;
    };

}

#endif /* URI_HPP */
