{
  description = "Rust development environment for kittygame (WASM-4)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
          extensions = [ "rust-src" "clippy" "rustfmt" "rust-analyzer" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.nodejs_22
            pkgs.binaryen
          ];

          shellHook = ''
            echo "⚡ Rust $(rustc --version) & WASM-4 dev environment loaded (NixOS 26.05)"
          '';
        };
      });
}
