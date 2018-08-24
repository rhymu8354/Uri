#ifndef URI_PERCENT_ENCODED_CHARACTER_DECODER_HPP
#define URI_PERCENT_ENCODED_CHARACTER_DECODER_HPP

/**
 * @file PercentEncodedCharacterDecoder.hpp
 *
 * This module declares the Uri::PercentEncodedCharacterDecoder class.
 *
 * © 2018 by Richard Walters
 */

#include <memory>
#include <stddef.h>

namespace Uri {

    /**
     * This class can take in a percent-encoded character,
     * decode it, and also detect if there are any problems in the encoding.
     */
    class PercentEncodedCharacterDecoder {
        // Lifecycle management
    public:
        ~PercentEncodedCharacterDecoder() noexcept;
        PercentEncodedCharacterDecoder(const PercentEncodedCharacterDecoder&) = delete;
        PercentEncodedCharacterDecoder(PercentEncodedCharacterDecoder&&) noexcept;
        PercentEncodedCharacterDecoder& operator=(const PercentEncodedCharacterDecoder&) = delete;
        PercentEncodedCharacterDecoder& operator=(PercentEncodedCharacterDecoder&&) noexcept;

        // Methods
    public:
        /**
         * This is the default constructor.
         */
        PercentEncodedCharacterDecoder();

        /**
         * This method inputs the next encoded character.
         *
         * @param[in] c
         *     This is the next encoded character to give to the decoder.
         *
         * @return
         *     An indication of whether or not the encoded character
         *     was accepted is returned.
         */
        bool NextEncodedCharacter(char c);

        /**
         * This method checks to see if the decoder is done
         * and has decoded the encoded character.
         *
         * @return
         *     An indication of whether or not the decoder is done
         *     and has decoded the encoded character is returned.
         */
        bool Done() const;

        /**
         * This method returns the decoded character, once
         * the decoder is done.
         *
         * @return
         *     The decoded character is returned.
         */
        char GetDecodedCharacter() const;

        // Properties
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

#endif /* URI_PERCENT_ENCODED_CHARACTER_DECODER_HPP */
