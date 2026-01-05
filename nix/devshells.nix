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

      devInputs = [
        rustToolchains.rustToolchainDevTOML
        pkgs.just
        pkgs.cargo-watch
        pkgs.cargo-insta
      ];

      msrvDevInputs = [ rustToolchains.rustToolchainMSRV ];

      lintInputs = [
        pkgs.reuse
        pkgs.lychee
        pkgs.typos
        pkgs.taplo
        pkgs.cargo-deny
        pkgs.cargo-diet
        pkgs.cargo-dist
        pkgs.cargo-flamegraph
        pkgs.cargo-machete
        pkgs.cargo-modules
        pkgs.cargo-outdated
        pkgs.cargo-tarpaulin
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
        (pkgs.symlinkJoin {
          name = "cargo-careful-wrapped";
          paths = [ pkgs.cargo-careful ];
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram $out/bin/cargo-careful \
              --prefix PATH : ${
                pkgs.lib.makeBinPath [
                  (rustToolchains.rustPkgs.rust-bin.selectLatestNightlyWith (
                    toolchain: toolchain.default.override { extensions = [ "rust-src" ]; }
                  ))
                ]
              }
          '';
        })
        (pkgs.symlinkJoin {
          name = "cargo-public-api-wrapped";
          paths = [ pkgs.cargo-public-api ];
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram $out/bin/cargo-public-api \
              --prefix PATH : ${
                pkgs.lib.makeBinPath [
                  (rustToolchains.rustPkgs.rust-bin.selectLatestNightlyWith (
                    toolchain: toolchain.default.override { extensions = [ "rust-src" ]; }
                  ))
                ]
              }
          '';
        })
      ];

      shellInputs = [
        pkgs.shellcheck
        pkgs.actionlint
      ];

      editorConfigInputs = [ pkgs.editorconfig-checker ];
      actionlintInputs = [ pkgs.actionlint ];
    in
    {
      devShells = {
        default = pkgs.mkShell {
          buildInputs = shellInputs ++ devInputs;
          packages = [ self'.formatter.outPath ];
          inherit (tuiTerm) name;
          RUST_BACKTRACE = true;
        };

        msrvShell = pkgs.mkShell {
          buildInputs = msrvDevInputs;
          name = "msrvShell";
          RUST_BACKTRACE = true;
        };

        editorConfigShell = pkgs.mkShell {
          buildInputs = editorConfigInputs;
        };

        actionlintShell = pkgs.mkShell {
          buildInputs = actionlintInputs;
        };

        lintShell = pkgs.mkShell {
          buildInputs = lintInputs;
        };
      };
    };
}
