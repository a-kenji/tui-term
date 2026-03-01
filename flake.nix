{
  description = "tui-term - a pseudoterminal widget for ratatui";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane = {
      # Pin before https://github.com/ipetkov/crane/pull/976 which requires
      # unstable cargo flag --exclude-lockfile
      url = "github:ipetkov/crane/v0.23.0";
    };
  };

  outputs = args: import ./nix args;
}
