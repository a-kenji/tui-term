{ inputs, ... }:
{
  imports = [ inputs.treefmt-nix.flakeModule ];

  perSystem = _: {
    treefmt = {
      projectRootFile = ".git/config";
      programs.actionlint.enable = true;
      programs.flake-edit.enable = true;
      programs.nixfmt.enable = true;
      programs.rustfmt.enable = true;
      programs.shellcheck.enable = true;
      programs.shfmt.enable = true;
      programs.taplo.enable = true;

      settings.formatter.rustfmt.options = [
        "--config"
        "newline_style=Unix"
      ];
    };
  };
}
