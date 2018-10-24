/**
 * @file PercentEncodedCharacterDecoderTests.cpp
 *
 * This module contains the unit tests of the Uri::PercentEncodedCharacterDecoder class.
 *
 * © 2018 by Richard Walters
 */

#include <gtest/gtest.h>
#include <src/PercentEncodedCharacterDecoder.hpp>
#include <stddef.h>
#include <vector>

TEST(PercentEncodedCharacterDecoderTests, GoodSequences) {
    Uri::PercentEncodedCharacterDecoder pec;
    struct TestVector {
        char sequence[2];
        char expectedOutput;
    };
    const std::vector< TestVector > testVectors{
        {{'4', '1'}, 'A'},
        {{'5', 'A'}, 'Z'},
        {{'6', 'e'}, 'n'},
        {{'e', '1'}, (char)0xe1},
        {{'C', 'A'}, (char)0xca},
    };
    size_t index = 0;
    for (auto testVector: testVectors) {
        pec = Uri::PercentEncodedCharacterDecoder();
        ASSERT_FALSE(pec.Done());
        ASSERT_TRUE(pec.NextEncodedCharacter(testVector.sequence[0]));
        ASSERT_FALSE(pec.Done());
        ASSERT_TRUE(pec.NextEncodedCharacter(testVector.sequence[1]));
        ASSERT_TRUE(pec.Done());
        ASSERT_EQ(testVector.expectedOutput, pec.GetDecodedCharacter()) << index;
        ++index;
    }
}

TEST(PercentEncodedCharacterDecoderTests, BadSequences) {
    Uri::PercentEncodedCharacterDecoder pec;
    std::vector< char > testVectors{
        'G', 'g', '.', 'z', '-', ' ', 'V',
    };
    for (auto testVector: testVectors) {
        pec = Uri::PercentEncodedCharacterDecoder();
        ASSERT_FALSE(pec.Done());
        ASSERT_FALSE(pec.NextEncodedCharacter(testVector));
    }
}
