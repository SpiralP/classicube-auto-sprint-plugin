{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
  };

  outputs = { nixpkgs, ... }:
    let
      inherit (nixpkgs) lib;

      makePackages = (system: dev:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in
        rec {
          default = pkgs.rustPlatform.buildRustPackage {
            name = "classicube-auto-sprint-plugin";
            src = lib.cleanSourceWith {
              src = ./.;
              filter = path: type:
                lib.cleanSourceFilter path type
                && (
                  lib.any (re: builtins.match re (lib.removePrefix (builtins.toString ./.) (builtins.toString path)) != null) [
                    "/Cargo.toml"
                    "/Cargo.lock"
                    "/\.cargo"
                    "/\.cargo/.*"
                    "/src"
                    "/src/.*"
                  ]
                );
            };

            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = { };
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
              rustPlatform.bindgenHook
            ] ++ (if dev then
              with pkgs; [
                clippy
                (rustfmt.override { asNightly = true; })
                rust-analyzer
              ] else [ ]);

            buildInputs = with pkgs; [
              openssl
            ];
          };
        }
      );
    in
    builtins.foldl' lib.recursiveUpdate { } (builtins.map
      (system: {
        devShells.${system} = makePackages system true;
        packages.${system} = makePackages system false;
      })
      lib.systems.flakeExposed);
}
