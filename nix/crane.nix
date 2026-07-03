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
      enableSystemd = pkgs.stdenv.hostPlatform.isLinux;
      commonArgs = {
        inherit src;
        strictDeps = true;
        __structuredAttrs = true;
        cargoExtraArgs = lib.optionalString (!enableSystemd) "--no-default-features";

        __darwinAllowLocalNetworking = true;

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        buildInputs = lib.optionals enableSystemd (
          with pkgs;
          [
            systemd
          ]
        );
      };
      src = lib.fileset.toSource {
        root = ./..;
        fileset = lib.fileset.unions [
          (craneLib.fileset.commonCargoSources ../.)
          ../resources
        ];
      };

      whois42d-ng = craneLib.buildPackage (
        commonArgs
        // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          nativeBuildInputs =
            with pkgs;
            (commonArgs.nativeBuildInputs or [ ])
            ++ [
              installShellFiles
            ];
          postInstall =
            lib.optionalString enableSystemd ''
              substituteInPlace resources/whois42d-ng.service \
                --replace-fail '/usr/local/bin' "$out/bin"
              install -Dm444 resources/whois42d-ng.service -t $out/lib/systemd/system
              install -Dm444 resources/whois42d-ng{,-rdap}.socket -t $out/lib/systemd/system
            ''
            + lib.optionalString (pkgs.stdenv.buildPlatform.canExecute pkgs.stdenv.hostPlatform) ''
              installShellCompletion --cmd whois42d-ng \
                --zsh <($out/bin/whois42d-ng completions zsh) \
                --bash <($out/bin/whois42d-ng completions bash) \
                --fish <($out/bin/whois42d-ng completions fish)
            '';
        }
      );
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
    in
    {
      _module.args = {
        packaging = {
          inherit
            cargoArtifacts
            commonArgs
            src
            ;
        };
        craneLib = inputs.crane.mkLib pkgs;
      };

      packages = {
        inherit whois42d-ng;
        default = config.packages.whois42d-ng;
      };
    };
}
