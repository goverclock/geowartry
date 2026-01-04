cmake_minimum_required(VERSION 3.24...3.30)
project(raylib-game-template)

# Generate compile_commands.json
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Dependencies
set(RAYLIB_VERSION 5.5)

include(FetchContent)
FetchContent_Declare(
    raylib
    DOWNLOAD_EXTRACT_TIMESTAMP OFF
    URL https://github.com/raysan5/raylib/archive/refs/tags/${RAYLIB_VERSION}.tar.gz
    FIND_PACKAGE_ARGS
)
FetchContent_MakeAvailable(raylib)

set(CMAKE_BUILD_TYPE Debug)
add_executable(${PROJECT_NAME})
add_subdirectory(src)

# maybe helpful when you want to fix "Failed to open file"
# add_custom_command(
# 	TARGET ${PROJECT_NAME} POST_BUILD
# 	COMMAND ${CMAKE_COMMAND} -E copy_directory ${CMAKE_SOURCE_DIR}/src/resources $<TARGET_FILE_DIR:${PROJECT_NAME}>/resources
# )

target_link_libraries(${PROJECT_NAME} raylib)

