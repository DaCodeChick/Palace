cmake_minimum_required(VERSION 3.21)
project(PalaceClient VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Qt setup
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

# Find Qt 6.10
find_package(Qt6 6.10 REQUIRED COMPONENTS
    Core
    Gui
    Quick
    Qml
    Network
    ShaderTools
)

# Build thepalace Rust library first
add_custom_target(thepalace_rust ALL
    COMMAND cargo build --release --manifest-path 
            ${CMAKE_SOURCE_DIR}/lib/thepalace/Cargo.toml
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    COMMENT "Building thepalace Rust library"
)

# Import thepalace as external library
add_library(thepalace SHARED IMPORTED)
if(WIN32)
    set(THEPALACE_LIB "${CMAKE_SOURCE_DIR}/target/release/thepalace.dll")
elseif(APPLE)
    set(THEPALACE_LIB "${CMAKE_SOURCE_DIR}/target/release/libthepalace.dylib")
else()
    set(THEPALACE_LIB "${CMAKE_SOURCE_DIR}/target/release/libthepalace.so")
endif()

set_target_properties(thepalace PROPERTIES
    IMPORTED_LOCATION ${THEPALACE_LIB}
)

# Include directories
include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_SOURCE_DIR}/lib/thepalace/include
)

# Source files
set(SOURCES
    src/main.cpp
    src/MainWindow.cpp
    src/network/Connection.cpp
    src/network/Protocol.cpp
    src/network/Session.cpp
    src/graphics/PalaceCanvas.cpp
    src/graphics/RhiRenderer.cpp
    src/graphics/SoftwareRenderer.cpp
    src/graphics/TextureCache.cpp
    src/ui/models/UserListModel.cpp
    src/ui/models/RoomListModel.cpp
    src/ui/models/ChatModel.cpp
    src/settings/Settings.cpp
    src/settings/GraphicsSettings.cpp
)

set(HEADERS
    src/MainWindow.h
    src/network/Connection.h
    src/network/Protocol.h
    src/network/Session.h
    src/graphics/PalaceCanvas.h
    src/graphics/RhiRenderer.h
    src/graphics/SoftwareRenderer.h
    src/graphics/TextureCache.h
    src/ui/models/UserListModel.h
    src/ui/models/RoomListModel.h
    src/ui/models/ChatModel.h
    src/settings/Settings.h
    src/settings/GraphicsSettings.h
)

# QML resources
qt6_add_resources(QML_RESOURCES
    resources/qml.qrc
)

# Executable
add_executable(palace-client
    ${SOURCES}
    ${HEADERS}
    ${QML_RESOURCES}
)

# Add dependency on Rust build
add_dependencies(palace-client thepalace_rust)

# Link libraries
target_link_libraries(palace-client
    Qt6::Core
    Qt6::Gui
    Qt6::Quick
    Qt6::Qml
    Qt6::Network
    thepalace
)

# Platform-specific settings
if(WIN32)
    # Windows-specific settings
    set_target_properties(palace-client PROPERTIES
        WIN32_EXECUTABLE TRUE
    )
elseif(APPLE)
    # macOS-specific settings
    set_target_properties(palace-client PROPERTIES
        MACOSX_BUNDLE TRUE
    )
endif()

# Install target
install(TARGETS palace-client
    BUNDLE DESTINATION .
    RUNTIME DESTINATION bin
)

# Copy thepalace library to output directory
if(WIN32)
    add_custom_command(TARGET palace-client POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${THEPALACE_LIB}
        $<TARGET_FILE_DIR:palace-client>
    )
endif()
