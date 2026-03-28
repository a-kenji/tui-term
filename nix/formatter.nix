{ inputs, ... }:
{
  imports = [ inputs.treefmt-nix.flakeModule ];

  perSystem = _: {
    treefmt = {
      projectRootFile = ".git/config";
      programs.nixfmt.enable = true;
      programs.shellcheck.enable = true;
      programs.shfmt.enable = true;
      programs.rustfmt.enable = true;
      programs.taplo.enable = true;

      settings.formatter.rustfmt.options = [
        "--config"
        "newline_style=Unix"
      ];
    };
  };
}
