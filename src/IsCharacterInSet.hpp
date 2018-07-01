#ifndef URI_IS_CHARACTER_IN_SET_HPP
#define URI_IS_CHARACTER_IN_SET_HPP

/**
 * @file IsCharacterInSet.hpp
 *
 * This module declares the Uri::IsCharacterInSet class.
 *
 * © 2018 by Richard Walters
 */

#include <initializer_list>

namespace Uri {

    /**
     * This function determines whether or not the given character
     * is in the given character set.
     *
     * @param[in] c
     *     This is the character to check.
     *
     * @param[in] characterSet
     *     This is the set of characters that are allowed.
     *
     * @return
     *     An indication of whether or not the given character
     *     is in the given character set is returned.
     */
    bool IsCharacterInSet(
        char c,
        std::initializer_list< char > characterSet
    );

}

#endif /* URI_IS_CHARACTER_IN_SET_HPP */
