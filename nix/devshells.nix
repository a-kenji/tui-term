{self, ...}: {
  perSystem = {
    pkgs,
    system,
    ...
  }: let
    tuiTerm = pkgs.callPackage ./tui-term.nix {inherit self system;};
  in {
    devShells = {
      default = pkgs.mkShell {
        buildInputs = tuiTerm.shellInputs ++ tuiTerm.devInputs ++ tuiTerm.fmtInputs;
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

      fmtShell = pkgs.mkShell {
        buildInputs = tuiTerm.fmtInputs;
      };

      lintShell = pkgs.mkShell {
        buildInputs = tuiTerm.lintInputs;
      };
    };
  };
}
