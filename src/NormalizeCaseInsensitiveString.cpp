/**
 * @file NormalizeCaseInsensitiveString.cpp
 *
 * This module contains the implementation of the
 * Uri::NormalizeCaseInsensitiveString function.
 *
 * © 2018 by Richard Walters
 */

#include "NormalizeCaseInsensitiveString.hpp"

#include <ctype.h>

namespace Uri {

    std::string NormalizeCaseInsensitiveString(const std::string& inString) {
        std::string outString;
        for (char c: inString) {
            outString.push_back(tolower(c));
        }
        return outString;
    }

}
