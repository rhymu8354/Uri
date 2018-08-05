# CMakeLists.txt for UriTests
#
# © 2018 by Richard Walters

cmake_minimum_required(VERSION 3.8)
set(This UriTests)

set(Sources
    src/UriTests.cpp
    src/CharacterSetTests.cpp
    src/PercentEncodedCharacterDecoderTests.cpp
)

add_executable(${This} ${Sources})
set_target_properties(${This} PROPERTIES
    FOLDER Tests
)

target_include_directories(${This} PRIVATE ..)

target_link_libraries(${This} PUBLIC
    gtest_main
    Uri
)

add_test(
    NAME ${This}
    COMMAND ${This}
)
