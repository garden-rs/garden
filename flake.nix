{
  description = "Garden grows and cultivates collections of Git trees";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane = {
      url = "github:ipetkov/crane";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };
    flake-utils.url = "github:numtide/flake-utils";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, crane, fenix, flake-utils, advisory-db, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        inherit (pkgs) lib;

        craneLib = (crane.mkLib pkgs).overrideToolchain fenix.packages.${system}.stable.toolchain;

        # Include *.yaml and *.sh files for the test suite.
        src = lib.cleanSourceWith {
            src = craneLib.path ./.;  # Original, unfiltered sources
            filter = path: type:
                (craneLib.filterCargoSources path type)
                || (builtins.match ".*\\.sh$" path != null)
                || (builtins.match ".*\\.yaml$" path != null);
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          nativeBuildInputs = [
            pkgs.git
          ];

          buildInputs = [
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            # Darwin-specific inputs
            pkgs.libiconv
          ];
        };
        craneLibLLvmTools = craneLib.overrideToolchain
          (fenix.packages.${system}.complete.withComponents [
            "cargo"
            "rustc"
            "llvm-tools"
          ]);

        # Build cargo dependencies to be reused via cachix when running in CI.
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        # Build the crate while reusing the dependency artifacts from above.
        garden = craneLib.buildPackage (commonArgs // {
          pname = "garden";
          inherit cargoArtifacts;
        });
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit garden;

          garden-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          garden-doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
          });

          garden-fmt = craneLib.cargoFmt {
            inherit src;
          };

          garden-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          garden-deny = craneLib.cargoDeny {
            inherit src;
          };

          garden-nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
        };

        packages = {
          default = garden;
          garden-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs // {
            inherit cargoArtifacts;
          });

        };

        apps.default = flake-utils.lib.mkApp {
          drv = garden;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages = [
            garden
            pkgs.mdbook
          ];
        };
      });
}
