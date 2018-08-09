/**
 * @file PercentEncodedCharacterDecoder.cpp
 *
 * This module contains the implementation of the
 * Uri::PercentEncodedCharacterDecoder class.
 *
 * © 2018 by Richard Walters
 */

#include "CharacterSet.hpp"
#include "PercentEncodedCharacterDecoder.hpp"

namespace {

    /**
     * This is the character set containing just numbers.
     */
    const Uri::CharacterSet DIGIT('0', '9');

    /**
     * This is the character set containing just the upper-case
     * letters 'A' through 'F', used in upper-case hexadecimal.
     */
    const Uri::CharacterSet HEX_UPPER('A', 'F');

    /**
     * This is the character set containing just the lower-case
     * letters 'a' through 'f', used in lower-case hexadecimal.
     */
    const Uri::CharacterSet HEX_LOWER('a', 'f');

}

namespace Uri {

    struct PercentEncodedCharacterDecoder::Impl {
        // Properties

        /**
         * This is the decoded character.
         */
        int decodedCharacter = 0;

        /**
         * This is the number of digits that we still need to shift in
         * to decode the character.
         */
        size_t digitsLeft = 2;

        // Methods

        /**
         * This method shifts in the given hex digit as part of
         * building the decoded character.
         *
         * @param[in] c
         *     This is the hex digit to shift into the decoded character.
         *
         * @return
         *     An indication of whether or not the given hex digit
         *     was valid is returned.
         */
        bool ShiftInHexDigit(char c) {
            decodedCharacter <<= 4;
            if (DIGIT.Contains(c)) {
                decodedCharacter += (int)(c - '0');
            } else if (HEX_UPPER.Contains(c)) {
                decodedCharacter += (int)(c - 'A') + 10;
            } else if (HEX_LOWER.Contains(c)) {
                decodedCharacter += (int)(c - 'a') + 10;
            } else {
                return false;
            }
            return true;
        }
    };

    PercentEncodedCharacterDecoder::~PercentEncodedCharacterDecoder() noexcept = default;
    PercentEncodedCharacterDecoder::PercentEncodedCharacterDecoder(PercentEncodedCharacterDecoder&&) noexcept = default;
    PercentEncodedCharacterDecoder& PercentEncodedCharacterDecoder::operator=(PercentEncodedCharacterDecoder&&) noexcept = default;

    PercentEncodedCharacterDecoder::PercentEncodedCharacterDecoder()
        : impl_(new Impl)
    {
    }

    bool PercentEncodedCharacterDecoder::NextEncodedCharacter(char c) {
        if (!impl_->ShiftInHexDigit(c)) {
            return false;
        }
        --impl_->digitsLeft;
        return true;
    }

    bool PercentEncodedCharacterDecoder::Done() const {
        return (impl_->digitsLeft == 0);
    }

    char PercentEncodedCharacterDecoder::GetDecodedCharacter() const {
        return (char)impl_->decodedCharacter;
    }

}
