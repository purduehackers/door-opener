# Compat shim: delegates to flake.nix devShell.
# Use `nix develop` (flake) or `nix-shell` (legacy) interchangeably.
(builtins.getFlake (toString ./.)).devShells.${builtins.currentSystem}.default
