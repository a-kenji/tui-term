{ self, ... }:
{
  perSystem =
    {
      pkgs,
      system,
      self',
      ...
    }:
    let
      tuiTerm = pkgs.callPackage ./tui-term.nix { inherit self system; };
      rustToolchains = pkgs.callPackage ./rust-toolchain.nix { inherit self system; };
    in
    {
      devShells = {
        default = pkgs.mkShell {
          buildInputs = [
            rustToolchains.rustToolchainDevTOML
            pkgs.just
            pkgs.cargo-insta
          ];
          packages = [ self'.formatter.outPath ];
          inherit (tuiTerm) name;
        };

        msrvShell = pkgs.mkShell {
          buildInputs = [ rustToolchains.rustToolchainMSRV ];
          name = "MSRV";
        };

        ciShell = pkgs.mkShell {
          buildInputs = [
            pkgs.actionlint
            pkgs.shellcheck
            pkgs.cargo-deny
            pkgs.editorconfig-checker
            pkgs.lychee
            pkgs.typos
            (pkgs.symlinkJoin {
              name = "cargo-udeps-wrapped";
              paths = [ pkgs.cargo-udeps ];
              nativeBuildInputs = [ pkgs.makeWrapper ];
              postBuild = ''
                wrapProgram $out/bin/cargo-udeps \
                  --prefix PATH : ${
                    pkgs.lib.makeBinPath [
                      (rustToolchains.rustPkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default))
                    ]
                  }
              '';
            })
          ];
        };
      };
    };
}
