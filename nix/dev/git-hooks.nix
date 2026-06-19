{ inputs, ... }:
{
  imports = [ inputs.git-hooks-nix.flakeModule ];

  perSystem = {
    pre-commit = {
      check.enable = true;
      settings.hooks = {
        # formatter
        treefmt = {
          enable = true;
          settings = {
            fail-on-change = false;
          };
        };
        # security
        trufflehog.enable = true;
        # nix
        flake-checker.enable = true;
        flake-check = {
          enable = true;
          name = "nix flake check";
          entry = "nix flake check";
          language = "system";
          pass_filenames = false;
          stages = [ "pre-push" ];
        };
      };
    };
  };
}
