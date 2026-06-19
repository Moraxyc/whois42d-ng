{ inputs, ... }:
{
  perSystem =
    {
      config,
      craneLib,
      packaging,
      ...
    }:
    {
      checks = {
        inherit (config.packages) whois42d-ng;
        whois42d-ng-audit = craneLib.cargoAudit {
          inherit (packaging) src;
          inherit (inputs) advisory-db;
        };
        whois42d-ng-clippy = craneLib.cargoClippy (
          packaging.commonArgs
          // {
            inherit (packaging) cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          }
        );
        whois42d-ng-nextest = craneLib.cargoNextest (
          packaging.commonArgs
          // {
            inherit (packaging) cargoArtifacts;
            partitions = 1;
            partitionType = "count";
            cargoNextestPartitionsExtraArgs = "--no-tests=pass";
          }
        );
      };
    };
}
