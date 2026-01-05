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
      inherit
        (self'.checks)
        simple_ls_chan
        simple_ls_rw
        smux
        long_running
        nested_shell
        nested_shell_async
        cargoArtifacts
        cargoArtifactsMSRV
        cargoNextest
        cargoDoc
        ;
      default = self'.checks.smux;
    };
  };
}
