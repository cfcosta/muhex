{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
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
      pre-commit-hooks,
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
                rustfmt.package = rust;
              };
            };

            programs = {
              nixfmt.enable = true;
              taplo.enable = true;
              rustfmt.enable = true;
            };
          }).config.build.wrapper;

        pre-commit-check = pre-commit-hooks.lib.${system}.run {
          src = ./.;

          hooks = {
            deadnix.enable = true;
            nixfmt-rfc-style.enable = true;
            treefmt = {
              enable = true;
              package = muhex-formatter;
            };
          };
        };

        packages.default = rustPlatform.buildRustPackage {
          name = "muhex";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      in
      {
        inherit packages;

        formatter = muhex-formatter;

        checks = { inherit pre-commit-check; };

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
