{ inputs, ... }:
{
  perSystem =
    {
      lib,
      config,
      craneLib,
      pkgs,
      ...
    }:
    let
      src = lib.fileset.toSource {
        root = ./..;
        fileset = lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ../.)
          ../resources
        ];
      };
      mkPackage =
        packagePkgs:
        let
          packageCraneLib = inputs.crane.mkLib packagePkgs;
          enableSystemd = packagePkgs.stdenv.hostPlatform.isLinux;
          commonArgs = {
            inherit src;
            strictDeps = true;
            __structuredAttrs = true;
            cargoExtraArgs = lib.optionalString (!enableSystemd) "--no-default-features";
            __darwinAllowLocalNetworking = true;
          };
          cargoArtifacts = packageCraneLib.buildDepsOnly commonArgs;
          package = packageCraneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              nativeBuildInputs = [ packagePkgs.installShellFiles ];
              postInstall =
                lib.optionalString enableSystemd ''
                  substituteInPlace resources/whois42d-ng.service \
                    --replace-fail '/usr/local/bin' "$out/bin"
                  install -Dm444 resources/whois42d-ng.service -t $out/lib/systemd/system
                  install -Dm444 resources/whois42d-ng{,-rdap}.socket -t $out/lib/systemd/system
                ''
                + lib.optionalString (packagePkgs.stdenv.buildPlatform.canExecute packagePkgs.stdenv.hostPlatform) ''
                  installShellCompletion --cmd whois42d-ng \
                    --zsh <($out/bin/whois42d-ng completions zsh) \
                    --bash <($out/bin/whois42d-ng completions bash) \
                    --fish <($out/bin/whois42d-ng completions fish)
                '';
            }
          );
        in
        {
          inherit
            commonArgs
            cargoArtifacts
            package
            src
            ;
        };
      packaging = mkPackage pkgs;
      staticPackaging = mkPackage pkgs.pkgsStatic;
    in
    {
      _module.args = {
        inherit packaging;
        craneLib = inputs.crane.mkLib pkgs;
      };

      packages = {
        whois42d-ng = packaging.package;
        default = config.packages.whois42d-ng;
        whois42d-ng-static =
          (if pkgs.stdenv.hostPlatform.isLinux then staticPackaging.package else packaging.package)
          .overrideAttrs
            {
              stripAllList = [ "bin" ];
            };
      }
      // lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
        whois42d-ng-deb = pkgs.callPackage ./deb.nix {
          inherit (config.packages) whois42d-ng-static;
        };
      };
    };
}
