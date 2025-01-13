{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      treefmt-nix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        inherit (pkgs) mkShell;

        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        rustPlatform = pkgs.makeRustPlatform {
          rustc = rust;
          cargo = rust;
        };

        muhex-formatter =
          (treefmt-nix.lib.evalModule pkgs {
            projectRootFile = "flake.nix";

            settings = {
              allow-missing-formatter = true;
              verbose = 0;

              global.excludes = [ "*.lock" ];

              formatter = {
                nixfmt.options = [ "--strict" ];

                rustfmt = {
                  package = rust;

                  options = [
                    "--config-path"
                    (toString ./rustfmt.toml)
                  ];
                };
              };
            };

            programs = {
              nixfmt.enable = true;
              taplo.enable = true;
              rustfmt.enable = true;
            };
          }).config.build.wrapper;

        packages.default = rustPlatform.buildRustPackage {
          name = "muhex";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      in
      {
        inherit packages;

        formatter = muhex-formatter;

        devShells.default = mkShell {
          name = "muhex";

          buildInputs = with pkgs; [
            rust
            muhex-formatter

            cargo-nextest
            cargo-watch
          ];
        };
      }
    );
}
