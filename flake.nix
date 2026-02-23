{
  description = "door-opener cross-platform dev environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
          targets = [
            "x86_64-unknown-linux-gnu"
            "aarch64-unknown-linux-gnu"
            "x86_64-pc-windows-gnu"
            "aarch64-apple-darwin"
            "x86_64-apple-darwin"
          ];
        };

        # Shared native build inputs (needed on all platforms)
        sharedNativeBuildInputs = with pkgs; [
          rust-toolchain
          pkg-config
          cmake # needed by aws-lc-sys
          perl  # needed by aws-lc-sys
          clang
          llvm
        ];

        # Linux-specific dependencies
        linuxBuildInputs = with pkgs; [
          dbus
          systemd
          xorg.libX11
          xorg.libXi
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXinerama
          libGL
          wayland
          libxkbcommon
          libnfc
        ];

        # Darwin-specific dependencies
        darwinBuildInputs = with pkgs; [
          apple-sdk_15
          libiconv
          libnfc
        ];

        isLinux = pkgs.stdenv.isLinux;
        isDarwin = pkgs.stdenv.isDarwin;

        platformBuildInputs =
          pkgs.lib.optionals isLinux linuxBuildInputs
          ++ pkgs.lib.optionals isDarwin darwinBuildInputs;

        # Cross-compilation toolchains for Linux targets
        crossPkgsLinuxGnu = if isLinux then
          import nixpkgs {
            inherit overlays;
            localSystem = system;
            crossSystem = { config = "x86_64-unknown-linux-gnu"; };
          }
        else null;

        crossPkgsLinuxAarch64 = if isLinux then
          import nixpkgs {
            inherit overlays;
            localSystem = system;
            crossSystem = { config = "aarch64-unknown-linux-gnu"; };
          }
        else null;

        crossPkgsWindows = if isLinux then
          import nixpkgs {
            inherit overlays;
            localSystem = system;
            crossSystem = { config = "x86_64-w64-mingw32"; };
          }
        else null;
      in {
        devShells = {
          # Default dev shell for native builds
          default = pkgs.mkShell {
            nativeBuildInputs = sharedNativeBuildInputs;
            buildInputs = platformBuildInputs;

            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

            shellHook = ''
              echo "door-opener dev shell (${system})"
              echo "Rust: $(rustc --version)"
              echo ""
              echo "Available shells:"
              echo "  nix develop        — native build (current platform)"
              ${pkgs.lib.optionalString isLinux ''
                echo "  nix develop .#cross-x86_64-linux    — cross-compile to x86_64 Linux"
                echo "  nix develop .#cross-aarch64-linux   — cross-compile to aarch64 Linux"
                echo "  nix develop .#cross-windows         — cross-compile to x86_64 Windows"
              ''}
              ${pkgs.lib.optionalString isDarwin ''
                echo "  cargo build --target aarch64-apple-darwin  — build for Apple Silicon"
                echo "  cargo build --target x86_64-apple-darwin   — build for Intel Mac"
              ''}
            '';
          };
        }
        # Cross-compilation shells (Linux host only)
        // pkgs.lib.optionalAttrs isLinux {
          cross-x86_64-linux = pkgs.mkShell {
            nativeBuildInputs = sharedNativeBuildInputs ++ [
              crossPkgsLinuxGnu.stdenv.cc
            ];
            buildInputs = with crossPkgsLinuxGnu; [
              dbus
              systemdLibs
              xorg.libX11
              xorg.libXi
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXinerama
              libGL
              wayland
              libxkbcommon
              libnfc
            ];

            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            CARGO_BUILD_TARGET = "x86_64-unknown-linux-gnu";

            shellHook = ''
              echo "door-opener cross-compile shell: x86_64-unknown-linux-gnu"
            '';
          };

          cross-aarch64-linux = pkgs.mkShell {
            nativeBuildInputs = sharedNativeBuildInputs ++ [
              crossPkgsLinuxAarch64.stdenv.cc
            ];
            buildInputs = with crossPkgsLinuxAarch64; [
              dbus
              systemdLibs
              xorg.libX11
              xorg.libXi
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXinerama
              libGL
              wayland
              libxkbcommon
              libnfc
            ];

            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            CARGO_BUILD_TARGET = "aarch64-unknown-linux-gnu";
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER =
              "${crossPkgsLinuxAarch64.stdenv.cc}/bin/aarch64-unknown-linux-gnu-cc";

            shellHook = ''
              echo "door-opener cross-compile shell: aarch64-unknown-linux-gnu"
            '';
          };

          cross-windows = pkgs.mkShell {
            nativeBuildInputs = sharedNativeBuildInputs ++ [
              crossPkgsWindows.stdenv.cc
              pkgs.wineWow64Packages.minimal
            ];

            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
            CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER =
              "${crossPkgsWindows.stdenv.cc}/bin/x86_64-w64-mingw32-cc";
            CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER = "wine64";

            shellHook = ''
              echo "door-opener cross-compile shell: x86_64-pc-windows-gnu"
              echo "Use 'wine64' to test binaries"
            '';
          };
        };

        # Checks — run `nix flake check` to verify the env evaluates
        checks.devShell = self.devShells.${system}.default;
      }
    );
}
