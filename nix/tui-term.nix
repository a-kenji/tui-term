{
  self,
  lib,
  pkgs,
  system,
}:
let
  rustToolchains = pkgs.callPackage ./rust-toolchain.nix { inherit self system; };
  inherit (rustToolchains) rustToolchainDevTOML rustToolchainMSRV;

  cargoTOML = fromTOML (builtins.readFile (self + "/Cargo.toml"));
  inherit (cargoTOML.package) name version;

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

  root = ../.;
  commonArgs = {
    src = lib.fileset.toSource {
      inherit root;
      fileset = lib.fileset.unions [
        (craneLib.fileset.commonCargoSources root)
        (lib.fileset.maybeMissing (root + /test))
        (lib.fileset.maybeMissing (root + /src/snapshots))
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

  inherit name;
}
