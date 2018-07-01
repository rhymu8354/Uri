/**
 * @file IsCharacterInSet.cpp
 *
 * This module contains the implementation of the
 * Uri::IsCharacterInSet function.
 *
 * © 2018 by Richard Walters
 */

#include "IsCharacterInSet.hpp"

namespace Uri {

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

}
