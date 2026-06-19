{ inputs, ... }:
{
  imports = [ inputs.treefmt-nix.flakeModule ];

  perSystem =
    { pkgs, lib, ... }:
    {
      treefmt = {
        projectRootFile = "LICENSE";
        programs = {
          # nix
          nixfmt.enable = true;
          # sh
          shellcheck.enable = true;
          # toml
          taplo = {
            enable = true;
            priority = 2; # cargo-sort messes up the indentation, so make sure to run taplo after it.
          };
          # rs
          rustfmt.enable = true;
        };
        settings.formatter = {
          taplo.excludes = [ "Cargo.toml" ];
          cargo-sort = {
            command = lib.getExe pkgs.cargo-sort;
            includes = [ "**/Cargo.toml" ];
            options = [ "--no-format" ];
            priority = 1;
          };
        };
      };
    };
}
