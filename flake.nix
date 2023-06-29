{
  description = "tui-term - a pseudoterminal widget for ratatui";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
    inputs.flake-utils.follows = "flake-utils";
  };

  inputs.crane = {
    url = "github:ipetkov/crane";
    inputs.nixpkgs.follows = "nixpkgs";
    inputs.rust-overlay.follows = "rust-overlay";
    inputs.flake-utils.follows = "flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
  }:
    flake-utils.lib.eachDefaultSystem
    (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      stdenv =
        if pkgs.stdenv.isLinux
        then pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv
        else pkgs.stdenv;
      overlays = [(import rust-overlay)];
      rustPkgs = import nixpkgs {
        inherit system overlays;
      };
      src = self;
      RUST_TOOLCHAIN = src + "/rust-toolchain.toml";
      cargoTOML = builtins.fromTOML (builtins.readFile (src + "/Cargo.toml"));
      inherit (cargoTOML.package) name rust-version version;
      rustToolchainTOML = rustPkgs.rust-bin.fromRustupToolchainFile RUST_TOOLCHAIN;
      rustToolchainDevTOML = rustToolchainTOML.override {
        extensions = ["rustfmt" "clippy" "rust-analysis" "rust-docs"];
        targets = [];
      };
      rustToolchainMSRV = rustPkgs.rust-bin.stable.${rust-version}.default.override {
        extensions = ["rustfmt" "clippy" "rust-analysis" "rust-docs"];
        targets = [];
      };

      # the example targets
      examples = ["simple_ls_chan" "simple_ls_rw" "smux" "long_running" "nested_shell" "nested_shell_async"];

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainTOML;
      craneLibMSRV = (crane.mkLib pkgs).overrideToolchain rustToolchainMSRV;
      mkExample = {example, ...}:
        craneLib.buildPackage (commonArgs
          // {
            inherit cargoArtifacts stdenv;
            pname = example;
            cargoExtraArgs = "--example ${example}";
            # Prevent cargo test and nextest from duplicating tests
            doCheck = false;
          });
      # Common arguments for the crane build
      commonArgs = {
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        inherit stdenv version;
      };
      # Build *just* the cargo dependencies, so we can reuse
      # all of that work (e.g. via cachix) when running in CI
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      cargoArtifactsMSRV = craneLibMSRV.buildDepsOnly commonArgs;
      cargoNextest = craneLib.cargoNextest {
        inherit cargoArtifacts src;
        partitions = 1;
        partitionType = "count";
      };
      cargoDoc = craneLib.cargoDoc (commonArgs
        // {
          inherit cargoArtifacts;
        });

      devInputs = [
        rustToolchainDevTOML
        pkgs.just
        pkgs.cargo-watch

        # snapshot testing
        pkgs.cargo-insta

        #alternative linker
        pkgs.llvmPackages.bintools
        pkgs.mold
        pkgs.clang
      ];
      msrvDevInputs = [
        rustToolchainMSRV
      ];
      lintInputs = [
        pkgs.reuse
        pkgs.lychee
        pkgs.typos

        pkgs.cargo-deny
        pkgs.cargo-diet
        pkgs.cargo-dist
        pkgs.cargo-flamegraph
        pkgs.cargo-machete
        pkgs.cargo-modules
        pkgs.cargo-outdated
        pkgs.cargo-tarpaulin
        pkgs.cargo-unused-features
        (pkgs.symlinkJoin {
          name = "cargo-udeps-wrapped";
          paths = [pkgs.cargo-udeps];
          nativeBuildInputs = [pkgs.makeWrapper];
          postBuild = ''
            wrapProgram $out/bin/cargo-udeps \
              --prefix PATH : ${pkgs.lib.makeBinPath [
              (rustPkgs.rust-bin.selectLatestNightlyWith
                (toolchain: toolchain.default))
            ]}
          '';
        })
        (pkgs.symlinkJoin {
          name = "cargo-careful-wrapped";
          paths = [pkgs.cargo-careful];
          nativeBuildInputs = [pkgs.makeWrapper];
          postBuild = ''
            wrapProgram $out/bin/cargo-careful \
              --prefix PATH : ${pkgs.lib.makeBinPath [
              (rustPkgs.rust-bin.selectLatestNightlyWith
                (
                  toolchain:
                    toolchain
                    .default
                    .override {
                      extensions = ["rust-src"];
                    }
                ))
            ]}
          '';
        })
        (pkgs.symlinkJoin {
          name = "cargo-public-api-wrapped";
          paths = [pkgs.cargo-public-api];
          nativeBuildInputs = [pkgs.makeWrapper];
          postBuild = ''
            wrapProgram $out/bin/cargo-public-api \
              --prefix PATH : ${pkgs.lib.makeBinPath [
              (rustPkgs.rust-bin.selectLatestNightlyWith
                (
                  toolchain:
                    toolchain
                    .default
                    .override {
                      extensions = ["rust-src"];
                    }
                ))
            ]}
          '';
        })
      ];
      shellInputs = [
        pkgs.shellcheck
        pkgs.actionlint
      ];
      fmtInputs = [
        pkgs.alejandra
        pkgs.treefmt
      ];
      editorConfigInputs = [
        pkgs.editorconfig-checker
      ];
      actionlintInputs = [
        pkgs.actionlint
      ];
    in {
      devShells = {
        default = (pkgs.mkShell.override {inherit stdenv;}) {
          buildInputs = shellInputs ++ fmtInputs ++ devInputs;
          inherit name;
          RUST_BACKTRACE = true;
          RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=${pkgs.mold}/bin/mold -C target-cpu=native";
        };
        msrvShell = (pkgs.mkShell.override {inherit stdenv;}) {
          buildInputs = msrvDevInputs;
          name = "msrvShell";
          RUST_BACKTRACE = true;
          RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=${pkgs.mold}/bin/mold -C target-cpu=native";
        };
        editorConfigShell = pkgs.mkShell {
          buildInputs = editorConfigInputs;
        };
        actionlintShell = pkgs.mkShell {
          buildInputs = actionlintInputs;
        };
        fmtShell = pkgs.mkShell {
          buildInputs = fmtInputs;
        };
        lintShell = pkgs.mkShell {
          buildInputs = lintInputs;
        };
      };
      packages =
        {
          inherit cargoArtifacts cargoArtifactsMSRV cargoNextest cargoDoc;
          default = self.outputs.packages.${system}.smux;
        }
        // pkgs.lib.genAttrs examples (example:
          mkExample {
            inherit example cargoArtifacts craneLib;
          });
      formatter = pkgs.alejandra;
      apps.default = {
        type = "app";
        program = "${self.outputs.packages.${system}.smux}/bin/smux";
      };
    });
}
