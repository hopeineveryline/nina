{
  description = "Nina, a friendly Rust CLI/TUI for NixOS management";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        nina = pkgs.callPackage ./nina.nix { };
      in
      {
        packages = {
          default = nina;
          nina = nina;
        };

        apps = {
          default = {
            type = "app";
            program = "${nina}/bin/nina";
          };
          nina = {
            type = "app";
            program = "${nina}/bin/nina";
          };
        };

        checks = {
          nixos-vm-smoke = import ./nix/checks/nixos-vm-smoke.nix {
            inherit pkgs nina;
            src = ./.;
          };
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            pkg-config
            openssl
          ];
        };

        devShells.kiln = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            cargo-nextest
            git
            openssl
            pkg-config
            rustc
          ];

          shellHook = ''
            echo "nina kiln shell ready"
            echo "use ./scripts/kiln-fire.sh to run the local ../kiln harness"
          '';
        };
      });
}
