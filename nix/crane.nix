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
      commonArgs = {
        inherit src;
        strictDeps = true;
        __structuredAttrs = true;

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];
        buildInputs = with pkgs; [
          systemd
        ];
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
          postInstall = ''
            substituteInPlace resources/whois42d-ng.service \
              --replace-fail '/usr/local/bin' "$out/bin"
            install -Dm644 resources/whois42d-ng.{service,socket} -t $out/lib/systemd/system
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
