{
  perSystem =
    {
      config,
      pkgs,
      craneLib,
      ...
    }:
    {
      devShells.default = craneLib.devShell {
        inherit (config.pre-commit.settings) shellHook;
        nativeBuildInputs = config.pre-commit.settings.enabledPackages;
        packages = with pkgs; [
          rust-analyzer
          cargo-nextest
          cargo-edit
        ];
      };
    };
}
