# door-opener

🔑 Passport-based access control & door opener for Hack Night.

[Demo](https://youtu.be/31aA09y7xFA)

## Building/deployment

**Warning**: most of the code in this repository assumes that you have a user
account named `hackers` on your Pi.

This project can be built on the Raspbian OS Lite image. However, since the project
displays a GUI, you do need X11 installed (read below).

Due to the GPIO libraries used, this project should be built directly on the
door opener. SSH into the door opener, pull any changes, and run `cargo build --release`.
If you are a member of Purdue Hackers and would like to contribute to door opener,
contact an organizer for SSH details!

If you are imaging a new door opener, install the following dependencies:

- `libssl-dev`
- `libudev-dev`
- `libnfc-dev`
- `libclang-dev`
- `xinit` (for X11)

### Local development

If you are developing this repository locally, there are some additional packages
you will need to install.

#### For Windows

These dependencies are required by the `nfc1` crate.

Install LLVM and CMake:

```
winget install -e --id LLVM.LLVM
winget install -e --id Kitware.CMake
```