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
        # Crane builds as checks
        inherit (tuiTerm)
          cargoArtifacts
          cargoArtifactsMSRV
          cargoNextest
          cargoDoc
          cargoClippy
          ;

        # Example builds as checks
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
