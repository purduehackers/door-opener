cmake_minimum_required(VERSION 3.13)

set(PICO_SDK_FETCH_FROM_GIT on)

include(pico_sdk_import.cmake)

project(servo_emulator)

pico_sdk_init()

add_executable(servo_emulator
    src/main.c
)

target_link_libraries(servo_emulator
    pico_multicore
    pico_stdlib
    hardware_pwm
)

pico_enable_stdio_usb(servo_emulator 1)
pico_enable_stdio_uart(servo_emulator 0)

pico_add_extra_outputs(servo_emulator)
