{

  inputs = {
    naersk.url = "github:nmattia/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk {};
      in {

        defaultPackage = naersk-lib.buildPackage {
          src = ./.;
          buildInputs = with pkgs; [ pkg-config openssl ];
        };

        defaultApp = utils.mkApp {
            drv = self.defaultPackage."${system}";
        };

        devShell = with pkgs; mkShell {
          buildInputs = [ cargo rustc rustfmt pre-commit rustPackages.clippy openssl pkg-config ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };

      });

}
