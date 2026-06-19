{
  description = "whois42d-ng";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.flake-parts.flakeModules.partitions
        ./nix/crane.nix
        ./nix/overlay.nix
      ];
      partitions = {
        dev = {
          module = ./nix/dev/flake-module.nix;
          extraInputsFlake = ./nix/dev;
        };
      };
      partitionedAttrs = {
        checks = "dev";
        devShells = "dev";
        formatter = "dev";
      };
      systems = import inputs.systems;
    };
}
