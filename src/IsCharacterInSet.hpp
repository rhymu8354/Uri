#ifndef URI_IS_CHARACTER_IN_SET_HPP
#define URI_IS_CHARACTER_IN_SET_HPP

/**
 * @file IsCharacterInSet.hpp
 *
 * This module declares the Uri::IsCharacterInSet function
 * and the CharacterSet class.
 *
 * © 2018 by Richard Walters
 */

#include <initializer_list>
#include <memory>

namespace Uri {

    /**
     * This represents a set of characters which can be queried
     * to find out if a character is in the set or not.
     */
    class CharacterSet {
        // Lifecycle management
    public:
        ~CharacterSet();
        CharacterSet(const CharacterSet&);
        CharacterSet(CharacterSet&&);
        CharacterSet& operator=(const CharacterSet&);
        CharacterSet& operator=(CharacterSet&&);

        // Methods
    public:
        /**
         * This is the default constructor.
         */
        CharacterSet();

        /**
         * This constructs a character set that contains
         * just the given character.
         *
         * @param[in] c
         *     This is the only character to put in the set.
         */
        CharacterSet(char c);

        /**
         * This constructs a character set that contains all the
         * characters between the given "first" and "last"
         * characters, inclusive.
         *
         * @param[in] first
         *     This is the first of the range of characters
         *     to put in the set.
         *
         * @param[in] last
         *     This is the last of the range of characters
         *     to put in the set.
         */
        CharacterSet(char first, char last);

        /**
         * This constructs a character set that contains all the
         * characters in all the other given character sets.
         *
         * @param[in] characterSets
         *     These are the character sets to include.
         */
        CharacterSet(
            std::initializer_list< const CharacterSet > characterSets
        );

        /**
         * This method checks to see if the given character
         * is in the character set.
         *
         * @param[in] c
         *     This is the character to check.
         *
         * @return
         *     An indication of whether or not the given character
         *     is in the character set is returned.
         */
        bool Contains(char c) const;

        // Private Properties
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
        const CharacterSet& characterSet
    );

}

#endif /* URI_IS_CHARACTER_IN_SET_HPP */
