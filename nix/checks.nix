{ self, ... }:
{
  perSystem =
    {
      pkgs,
      system,
      ...
    }:
    let
      tuiTerm = pkgs.callPackage ./tui-term.nix { inherit self system; };
    in
    {
      checks = {
        inherit (tuiTerm)
          cargoArtifacts
          cargoArtifactsMSRV
          cargoNextest
          cargoDoc
          cargoClippy
          ;

        inherit (tuiTerm.examplePackages)
          simple_ls_chan
          simple_ls_rw
          smux
          long_running
          nested_shell
          nested_shell_async
          ;
      };
    };
}
