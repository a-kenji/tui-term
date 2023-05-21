{
  description = "tui-term - a pseudoterminal widget for ratatui";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  inputs.rust-overlay = {
    url = "github:oxalica/rust-overlay";
    inputs.nixpkgs.follows = "nixpkgs";
    inputs.flake-utils.follows = "flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
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
      inherit (cargoTOML.package) version name;
      rustToolchainTOML = rustPkgs.rust-bin.fromRustupToolchainFile RUST_TOOLCHAIN;
      rustToolchainDevTOML = rustToolchainTOML.override {
        extensions = ["rustfmt" "clippy" "rust-analysis" "rust-docs"];
        targets = [];
      };
      gitDate = self.lastModifiedDate;
      gitRev = self.shortRev or "Not committed yet.";
      cargoLock = {
        lockFile = builtins.path {
          path = self + "/Cargo.lock";
          name = "Cargo.lock";
        };
        allowBuiltinFetchGit = true;
      };
      rustc = rustToolchainTOML;
      cargo = rustToolchainTOML;

      buildInputs = [
        pkgs.installShellFiles
        pkgs.sqlite
        pkgs.openssl
      ];
      nativeBuildInputs = [
        pkgs.pkg-config
      ];
      devInputs = [
        rustToolchainDevTOML
        pkgs.just
        pkgs.lychee

        pkgs.cargo-deny
        pkgs.cargo-bloat
        pkgs.cargo-watch
        pkgs.cargo-flamegraph
        pkgs.cargo-diet
        pkgs.cargo-modules
        pkgs.cargo-nextest
        pkgs.cargo-dist
        pkgs.cargo-public-api
        pkgs.cargo-unused-features

        # snapshot testing
        pkgs.cargo-insta

        pkgs.openssl # for `cargo xtask`

        # database cli
        pkgs.diesel-cli
        # tokio tui
        pkgs.tokio-console

        pkgs.reuse

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
        #alternative linker
        pkgs.llvmPackages.bintools
        pkgs.mold
        pkgs.clang
      ];
      shellInputs = [
        pkgs.shellcheck
        pkgs.actionlint
      ];
      fmtInputs = [
        pkgs.alejandra
        pkgs.treefmt
        pkgs.typos
      ];
      editorConfigInputs = [
        pkgs.editorconfig-checker
      ];
      actionlintInputs = [
        pkgs.actionlint
      ];
    in rec {
      devShells = {
        default = (pkgs.mkShell.override {inherit stdenv;}) {
          buildInputs = shellInputs ++ fmtInputs ++ devInputs ++ buildInputs ++ nativeBuildInputs;
          inherit name;
          RUST_BACKTRACE = true;
          RUSTFLAGS = "-C linker=clang -C link-arg=-fuse-ld=${pkgs.mold}/bin/mold -C target-cpu=native";
          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath buildInputs}"
          '';
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
      };
      # packages = {
      #   default =
      #     (
      #       pkgs.makeRustPlatform {
      #         inherit cargo rustc;
      #       }
      #     )
      #     .buildRustPackage {
      #       cargoDepsName = name;
      #       GIT_DATE = gitDate;
      #       GIT_REV = gitRev;
      #       doCheck = false;
      #       inherit
      #         name
      #         version
      #         src
      #         stdenv
      #         nativeBuildInputs
      #         buildInputs
      #         cargoLock
      #         ;
      #     };
      # };
      # apps.default = {
      #   type = "app";
      #   program = "${packages.default}/bin/${name}";
      # };
      formatter = pkgs.alejandra;
    });
}
