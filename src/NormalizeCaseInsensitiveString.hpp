#ifndef URI_NORMALIZE_CASE_INSENSITIVE_STRING_HPP
#define URI_NORMALIZE_CASE_INSENSITIVE_STRING_HPP

/**
 * @file NormalizeCaseInsensitiveString.hpp
 *
 * This module declares the Uri::NormalizeCaseInsensitiveString function.
 *
 * © 2018 by Richard Walters
 */

#include <string>

namespace Uri {

    /**
     * This function takes a string and swaps all upper-case characters
     * with their lower-case equivalents, returning the result.
     *
     * @param[in] inString
     *     This is the string to be normalized.
     *
     * @return
     *     The normalized string is returned.  All upper-case characters
     *     are replaced with their lower-case equivalents.
     */
    std::string NormalizeCaseInsensitiveString(const std::string& inString);
}

#endif /* URI_NORMALIZE_CASE_INSENSITIVE_STRING_HPP */
