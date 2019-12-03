/**
 * @file IsCharacterInSet.cpp
 *
 * This module contains the implementation of the
 * Uri::CharacterSet class.
 *
 * © 2018 by Richard Walters
 */

#include "CharacterSet.hpp"

#include <algorithm>
#include <set>

namespace Uri {

    /**
     * This contains the private properties of the CharacterSet class.
     */
    struct CharacterSet::Impl {
        /**
         * This holds the characters in the set.
         */
        std::set< char > charactersInSet;
    };

    CharacterSet::~CharacterSet() noexcept = default;
    CharacterSet::CharacterSet(const CharacterSet& other)
        : impl_(new Impl(*other.impl_))
    {
    }
    CharacterSet::CharacterSet(CharacterSet&& other) noexcept = default;
    CharacterSet& CharacterSet::operator=(const CharacterSet& other) {
        if (this != &other) {
            *impl_ = *other.impl_;
        }
        return *this;
    }
    CharacterSet& CharacterSet::operator=(CharacterSet&& other) noexcept = default;

    CharacterSet::CharacterSet()
        : impl_(new Impl)
    {
    }

    CharacterSet::CharacterSet(char c)
        : impl_(new Impl)
    {
        (void)impl_->charactersInSet.insert(c);
    }

    CharacterSet::CharacterSet(char first, char last)
        : impl_(new Impl)
    {
        if (first > last) {
            std::swap(first, last);
        }
        for (char c = first; c < last + 1; ++c) {
            (void)impl_->charactersInSet.insert(c);
        }
    }

    CharacterSet::CharacterSet(
        std::initializer_list< const CharacterSet > characterSets
    )
        : impl_(new Impl)
    {
        for (
            auto characterSet = characterSets.begin();
            characterSet != characterSets.end();
            ++characterSet
        ) {
            impl_->charactersInSet.insert(
                characterSet->impl_->charactersInSet.begin(),
                characterSet->impl_->charactersInSet.end()
            );
        }
    }

    bool CharacterSet::Contains(char c) const {
        return impl_->charactersInSet.find(c) != impl_->charactersInSet.end();
    }

}
