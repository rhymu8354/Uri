/**
 * @file PercentEncodedCharacterDecoder.cpp
 *
 * This module contains the implementation of the
 * Uri::PercentEncodedCharacterDecoder class.
 *
 * © 2018 by Richard Walters
 */

#include "IsCharacterInSet.hpp"
#include "PercentEncodedCharacterDecoder.hpp"

namespace Uri {

    struct PercentEncodedCharacterDecoder::Impl {
        /**
         * This is the decoded character.
         */
        int decodedCharacter = 0;

        /**
         * This is the current state of the decoder's state machine.
         * - 0: we haven't yet received the first hex digit.
         * - 1: we received the first hex digit but not the second.
         * - 2: we received both hex digits
         */
        size_t decoderState = 0;
    };

    PercentEncodedCharacterDecoder::~PercentEncodedCharacterDecoder() = default;
    PercentEncodedCharacterDecoder::PercentEncodedCharacterDecoder(PercentEncodedCharacterDecoder&&) = default;
    PercentEncodedCharacterDecoder& PercentEncodedCharacterDecoder::operator=(PercentEncodedCharacterDecoder&&) = default;

    PercentEncodedCharacterDecoder::PercentEncodedCharacterDecoder()
        : impl_(new Impl)
    {
    }

    bool PercentEncodedCharacterDecoder::NextEncodedCharacter(char c) {
        switch(impl_->decoderState) {
            case 0: { // % ...
                impl_->decoderState = 1;
                impl_->decodedCharacter <<= 4;
                if (IsCharacterInSet(c, {'0','9'})) {
                    impl_->decodedCharacter += (int)(c - '0');
                } else if (IsCharacterInSet(c, {'A','F'})) {
                    impl_->decodedCharacter += (int)(c - 'A') + 10;
                } else {
                    return false;
                }
            } break;

            case 1: { // %[0-9A-F] ...
                impl_->decoderState = 2;
                impl_->decodedCharacter <<= 4;
                if (IsCharacterInSet(c, {'0','9'})) {
                    impl_->decodedCharacter += (int)(c - '0');
                } else if (IsCharacterInSet(c, {'A','F'})) {
                    impl_->decodedCharacter += (int)(c - 'A') + 10;
                } else {
                    return false;
                }
            } break;

            default: break;
        }
        return true;
    }

    bool PercentEncodedCharacterDecoder::Done() const {
        return (impl_->decoderState == 2);
    }

    char PercentEncodedCharacterDecoder::GetDecodedCharacter() const {
        return (char)impl_->decodedCharacter;
    }

}
