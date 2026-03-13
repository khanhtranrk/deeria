{
  description = "Deeria";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, fenix, crane }:
  let
    system = "x86_64-linux";

    pkgs = import nixpkgs { inherit system; };

    toolchain =
      fenix.packages.${system}.stable.withComponents [
        "cargo"
        "clippy"
        "rustc"
        "rustfmt"
      ];

    craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

    src = ./.;

  in {

    packages.${system}.default =
      craneLib.buildPackage {
        inherit src;
      };

    apps.${system}.default = {
      type = "app";
      program = "${self.packages.${system}.default}/bin/deeria";
    };

    devShells.${system}.default = pkgs.mkShell {
      packages = [
        toolchain
        pkgs.rust-analyzer
      ];
    };
  };
}
