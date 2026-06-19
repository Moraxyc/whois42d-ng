{ inputs, self, ... }:
{
  imports = [ inputs.flake-parts.flakeModules.easyOverlay ];
  perSystem =
    { config, system, ... }:
    {
      overlayAttrs = { inherit (config.packages) whois42d-ng; };
      _module.args.pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ self.overlays.default ];
      };
    };
}
