# Local Development

If you are developing this repository locally, there are some additional packages
you will need to install.

## For Windows

These dependencies are required by the `nfc1` crate.

Install LLVM and CMake:

```
winget install -e --id LLVM.LLVM
winget install -e --id Kitware.CMake
```

## For macOS

Install Homebrew, and then install the following dependencies:

```
brew install libnfc pkgconf
```
