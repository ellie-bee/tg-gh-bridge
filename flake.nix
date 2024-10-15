{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
  let
    system = "x86_64-linux";
    packageName = "tggh_bridge";
    pkgs = import nixpkgs { inherit system; };
    nativeBuildInputs = [
      pkgs.rustc
      pkgs.cargo
      pkgs.clippy
      pkgs.rustfmt
    ];
  in rec {
    defaultPackage.${system} = pkgs.stdenv.mkDerivation {
      pname = packageName;
      version = "0.1.0";
      src = self;

      buildInputs = [ pkgs.cargo ];

      installPhase = ''
        mkdir -p $out/bin
        install -m755 target/release/${packageName} $out/bin/
      '';

      buildPhase = ''
        cargo build --release
      '';
    };

    devShells.${system}.default = pkgs.mkShell { inherit nativeBuildInputs; };
  };
}

