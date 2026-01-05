{self, ...}: {
  perSystem = {
    pkgs,
    system,
    self',
    ...
  }: let
    tuiTerm = pkgs.callPackage ./tui-term.nix {inherit self system;};
  in {
    packages = {
      inherit (self'.checks) smux;
      default = self'.checks.smux;
    };
  };
}
