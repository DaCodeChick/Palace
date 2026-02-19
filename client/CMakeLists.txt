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

# Include directories
include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/src
)

# Source files
set(SOURCES
    src/main.cpp
    src/network/Connection.cpp
    src/network/Protocol.cpp
    src/network/Session.cpp
)

set(HEADERS
    src/network/Connection.h
    src/network/Protocol.h
    src/network/Session.h
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

# Link libraries
target_link_libraries(palace-client
    Qt6::Core
    Qt6::Gui
    Qt6::Quick
    Qt6::Qml
    Qt6::Network
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
