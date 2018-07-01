# CMakeLists.txt for Uri
#
# © 2018 by Richard Walters

cmake_minimum_required(VERSION 3.8)
set(This Uri)

set(Headers
    include/Uri/Uri.hpp
    src/IsCharacterInSet.hpp
    src/PercentEncodedCharacterDecoder.hpp
)

set(Sources
    src/IsCharacterInSet.cpp
    src/PercentEncodedCharacterDecoder.cpp
    src/Uri.cpp
)

add_library(${This} STATIC ${Sources} ${Headers})
set_target_properties(${This} PROPERTIES
    FOLDER Libraries
)

target_include_directories(${This} PUBLIC include)

add_subdirectory(test)
