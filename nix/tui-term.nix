{
  self,
  lib,
  pkgs,
  system,
}:
let
  # Apply rust-overlay to get access to rust-bin
  overlays = [ (import self.inputs.rust-overlay) ];
  rustPkgs = import self.inputs.nixpkgs { inherit system overlays; };

  # Toolchain paths
  RUST_TOOLCHAIN = self + "/rust-toolchain.toml";

  # Parse Cargo.toml
  cargoTOML = builtins.fromTOML (builtins.readFile (self + "/Cargo.toml"));
  inherit (cargoTOML.package) name rust-version version;
  pname = name;

  # Rust toolchains using rust-overlay
  rustToolchainTOML = rustPkgs.rust-bin.fromRustupToolchainFile RUST_TOOLCHAIN;

  rustToolchainDevTOML = rustToolchainTOML.override {
    extensions = [
      "clippy"
      "rust-analysis"
      "rust-docs"
    ];
    targets = [ ];
  };

  rustToolchainMSRV = rustPkgs.rust-bin.stable.${rust-version}.default.override {
    extensions = [
      "rustfmt"
      "clippy"
      "rust-analysis"
      "rust-docs"
    ];
    targets = [ ];
  };

  # Crane libraries with custom toolchains
  craneLib = (self.inputs.crane.mkLib pkgs).overrideToolchain rustToolchainDevTOML;
  craneLibMSRV = (self.inputs.crane.mkLib pkgs).overrideToolchain rustToolchainMSRV;

  examples = [
    "simple_ls_chan"
    "simple_ls_rw"
    "smux"
    "long_running"
    "nested_shell"
    "nested_shell_async"
  ];

  unfilteredRoot = ../.;
  commonArgs = {
    src = lib.fileset.toSource {
      root = unfilteredRoot;
      fileset = lib.fileset.unions [
        (craneLib.fileset.commonCargoSources unfilteredRoot)
        (lib.fileset.maybeMissing (unfilteredRoot + /test))
        (lib.fileset.maybeMissing (unfilteredRoot + /src/snapshots))
      ];
    };
    inherit version;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
  cargoArtifactsMSRV = craneLibMSRV.buildDepsOnly commonArgs;

  cargoNextest = craneLib.cargoNextest {
    inherit cargoArtifacts;
    src = commonArgs.src;
    partitions = 1;
    partitionType = "count";
    cargoNextestExtraArgs = "--features unstable";
  };

  cargoDoc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

  cargoClippy = craneLib.cargoClippy (commonArgs // { inherit cargoArtifacts; });

  mkExample =
    { example, ... }:
    craneLib.buildPackage (
      commonArgs
      // {
        inherit cargoArtifacts;
        pname = example;
        cargoExtraArgs = "--example ${example}";
        doCheck = false;
      }
    );

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

  msrvDevInputs = [ rustToolchainMSRV ];

  lintInputs = [
    pkgs.reuse
    pkgs.lychee
    pkgs.typos
    pkgs.taplo

    pkgs.cargo-deny
    pkgs.cargo-diet
    pkgs.cargo-dist
    pkgs.cargo-flamegraph
    pkgs.cargo-machete
    pkgs.cargo-modules
    pkgs.cargo-outdated
    pkgs.cargo-tarpaulin
    # pkgs.cargo-unused-features
    (pkgs.symlinkJoin {
      name = "cargo-udeps-wrapped";
      paths = [ pkgs.cargo-udeps ];
      nativeBuildInputs = [ pkgs.makeWrapper ];
      postBuild = ''
        wrapProgram $out/bin/cargo-udeps \
          --prefix PATH : ${
            pkgs.lib.makeBinPath [
              (rustPkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default))
            ]
          }
      '';
    })
    (pkgs.symlinkJoin {
      name = "cargo-careful-wrapped";
      paths = [ pkgs.cargo-careful ];
      nativeBuildInputs = [ pkgs.makeWrapper ];
      postBuild = ''
        wrapProgram $out/bin/cargo-careful \
          --prefix PATH : ${
            pkgs.lib.makeBinPath [
              (rustPkgs.rust-bin.selectLatestNightlyWith (
                toolchain: toolchain.default.override { extensions = [ "rust-src" ]; }
              ))
            ]
          }
      '';
    })
    (pkgs.symlinkJoin {
      name = "cargo-public-api-wrapped";
      paths = [ pkgs.cargo-public-api ];
      nativeBuildInputs = [ pkgs.makeWrapper ];
      postBuild = ''
        wrapProgram $out/bin/cargo-public-api \
          --prefix PATH : ${
            pkgs.lib.makeBinPath [
              (rustPkgs.rust-bin.selectLatestNightlyWith (
                toolchain: toolchain.default.override { extensions = [ "rust-src" ]; }
              ))
            ]
          }
      '';
    })
  ];

  shellInputs = [
    pkgs.shellcheck
    pkgs.actionlint
  ];

  editorConfigInputs = [ pkgs.editorconfig-checker ];
  actionlintInputs = [ pkgs.actionlint ];

  examplePackages = pkgs.lib.genAttrs examples (
    example: mkExample { inherit example cargoArtifacts craneLib; }
  );
in
{
  inherit
    cargoArtifacts
    cargoArtifactsMSRV
    cargoNextest
    cargoDoc
    cargoClippy
    ;

  inherit examplePackages;

  inherit
    devInputs
    msrvDevInputs
    lintInputs
    shellInputs
    editorConfigInputs
    actionlintInputs
    name
    ;
}
