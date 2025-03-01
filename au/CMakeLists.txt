cmake_minimum_required(VERSION 3.15)
project(ChorusExMachina)

set(CLAP_PLUGIN_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../target/bundled/chorus_ex_machina.clap")

# the following lines are required to build AU SDK on macOS
set(CMAKE_CXX_STANDARD 17)
enable_language(OBJC)
enable_language(OBJCXX)

# ensure clap-wrapper lives as a submodule next to this CMakeLists.txt
# https://github.com/free-audio/clap-wrapper/
set(CLAP_WRAPPER_DOWNLOAD_DEPENDENCIES ON)
add_subdirectory(clap-wrapper)

# create AUv2 target
set(AUV2_TARGET ${PROJECT_NAME}_auv2)
add_library(${AUV2_TARGET} MODULE)

# use clap-wrapper to populate the AUv2 target
target_add_auv2_wrapper(
        TARGET ${AUV2_TARGET}
        MACOS_EMBEDDED_CLAP_LOCATION ${CLAP_PLUGIN_PATH}
        
        OUTPUT_NAME "Chorus Ex Machina"
        BUNDLE_IDENTIFIER "com.github.peastman.ChorusExMachina"
        BUNDLE_VERSION "0.7.0"
        MANUFACTURER_NAME "Chorus Ex Machina"
        MANUFACTURER_CODE "CEMa"
        SUBTYPE_CODE "----"
        INSTRUMENT_TYPE "aufx"
)
