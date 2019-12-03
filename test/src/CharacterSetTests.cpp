/**
 * @file CharacterSetTests.cpp
 *
 * This module contains the unit tests of the Uri::CharacterSet class.
 *
 * © 2018 by Richard Walters
 */

#include <gtest/gtest.h>
#include <src/CharacterSet.hpp>
#include <utility>
#include <vector>

TEST(CharacterSetTests, DefaultConstructor) {
    Uri::CharacterSet cs;
    for (char c = 0; c < 0x7F; ++c) {
        ASSERT_FALSE(cs.Contains(c));
    }
}

TEST(CharacterSetTests, SingleCharacterConstructor) {
    Uri::CharacterSet cs('X');
    for (char c = 0; c < 0x7F; ++c) {
        if (c == 'X') {
            ASSERT_TRUE(cs.Contains(c));
        } else {
            ASSERT_FALSE(cs.Contains(c));
        }
    }
}

TEST(CharacterSetTests, RangeConstructor) {
    Uri::CharacterSet cs('A', 'G');
    for (char c = 0; c < 0x7F; ++c) {
        if (
            (c >= 'A')
            && (c <= 'G')
        ) {
            ASSERT_TRUE(cs.Contains(c));
        } else {
            ASSERT_FALSE(cs.Contains(c));
        }
    }
}

TEST(CharacterSetTests, Range_Constructor_Reversed) {
    Uri::CharacterSet cs('G', 'A');
    for (char c = 0; c < 0x7F; ++c) {
        if (
            (c >= 'A')
            && (c <= 'G')
        ) {
            ASSERT_TRUE(cs.Contains(c));
        } else {
            ASSERT_FALSE(cs.Contains(c));
        }
    }
}

TEST(CharacterSetTests, InitializerListConstructor) {
    Uri::CharacterSet cs1{'X'};
    for (char c = 0; c < 0x7F; ++c) {
        if (c == 'X') {
            ASSERT_TRUE(cs1.Contains(c));
        } else {
            ASSERT_FALSE(cs1.Contains(c));
        }
    }
    Uri::CharacterSet cs2{'A', 'G'};
    for (char c = 0; c < 0x7F; ++c) {
        if (
            (c == 'A')
            || (c == 'G')
        ) {
            ASSERT_TRUE(cs2.Contains(c));
        } else {
            ASSERT_FALSE(cs2.Contains(c));
        }
    }
    Uri::CharacterSet cs3{Uri::CharacterSet('f', 'i')};
    for (char c = 0; c < 0x7F; ++c) {
        if (
            (c >= 'f')
            && (c <= 'i')
        ) {
            ASSERT_TRUE(cs3.Contains(c));
        } else {
            ASSERT_FALSE(cs3.Contains(c));
        }
    }
    Uri::CharacterSet cs4{Uri::CharacterSet('a', 'c'), Uri::CharacterSet('f', 'i')};
    for (char c = 0; c < 0x7F; ++c) {
        if (
            (
                (c >= 'a')
                && (c <= 'c')
            )
            || (
                (c >= 'f')
                && (c <= 'i')
            )
        ) {
            ASSERT_TRUE(cs4.Contains(c));
        } else {
            ASSERT_FALSE(cs4.Contains(c));
        }
    }
    Uri::CharacterSet cs5{Uri::CharacterSet('a', 'c'), Uri::CharacterSet('x')};
    for (char c = 0; c < 0x7F; ++c) {
        if (
            (
                (c >= 'a')
                && (c <= 'c')
            )
            || (c == 'x')
        ) {
            ASSERT_TRUE(cs5.Contains(c));
        } else {
            ASSERT_FALSE(cs5.Contains(c));
        }
    }
}
