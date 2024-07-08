cmake_minimum_required(VERSION 3.24)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

project(Ready4L4s DESCRIPTION "Determine whether a client's connection is ready for L4S")

set(BLOCKING_PORT 8080 CACHE STRING "The number of the port to block." )
message(STATUS "Going to block non-ECN traffic on ${BLOCKING_PORT}.")

add_executable(ecn_block load.c)
target_link_libraries(ecn_block bpf xdp)

add_dependencies(ecn_block bpf)

add_custom_target(bpf
    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_OBJECTS:bpf_block> ${CMAKE_BINARY_DIR}/block.o
    DEPENDS bpf_block)

add_library(bpf_block OBJECT block.c)
target_compile_options(bpf_block PUBLIC -c -g -O2 -target bpf)
target_include_directories(bpf_block PUBLIC ${CMAKE_BINARY_DIR})

configure_file("config.h.in" "./config.h")