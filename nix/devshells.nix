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
    in
    {
      devShells = {
        default = pkgs.mkShell {
          buildInputs = tuiTerm.shellInputs ++ tuiTerm.devInputs;
          packages = [ self'.formatter.outPath ];
          inherit (tuiTerm) name;
          RUST_BACKTRACE = true;
        };

        msrvShell = pkgs.mkShell {
          buildInputs = tuiTerm.msrvDevInputs;
          name = "msrvShell";
          RUST_BACKTRACE = true;
        };

        editorConfigShell = pkgs.mkShell {
          buildInputs = tuiTerm.editorConfigInputs;
        };

        actionlintShell = pkgs.mkShell {
          buildInputs = tuiTerm.actionlintInputs;
        };

        lintShell = pkgs.mkShell {
          buildInputs = tuiTerm.lintInputs;
        };
      };
    };
}
