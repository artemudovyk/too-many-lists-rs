{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    let
      systems = inputs.nixpkgs.lib.systems.flakeExposed;
      eachSystem = inputs.nixpkgs.lib.genAttrs systems;
    in
    {
      formatter = eachSystem (
        system:
        let
          pkgs = inputs.nixpkgs.legacyPackages.${system};
          treefmtEval = inputs.treefmt-nix.lib.evalModule pkgs {
            projectRootFile = "flake.nix";
            programs.rustfmt.enable = true;
          };
        in
        treefmtEval.config.build.wrapper
      );

      devShells = eachSystem (
        system:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };

          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
              "clippy"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = [
              rustToolchain
              pkgs.cargo-nextest
            ];
          };
        }
      );
    };
}
