{
  self,
  system,
}:
let
  overlays = [ (import self.inputs.rust-overlay) ];
  rustPkgs = import self.inputs.nixpkgs { inherit system overlays; };

  RUST_TOOLCHAIN = self + "/rust-toolchain.toml";

  cargoTOML = fromTOML (builtins.readFile (self + "/Cargo.toml"));
  rust-version = cargoTOML.package.rust-version;

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
in
{
  inherit
    rustPkgs
    rustToolchainTOML
    rustToolchainDevTOML
    rustToolchainMSRV
    ;
}
