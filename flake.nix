{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
    rust-overlay,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      toolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
        toolchain.default.override {
          extensions = ["rust-src" "rust-analyzer"];
        });
      naersk-lib = pkgs.callPackage naersk {
        cargo = toolchain;
        rustc = toolchain;
      };
      darwin-frameworks = with pkgs; lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [Security SystemConfiguration]);
    in {
      packages.default = naersk-lib.buildPackage {
        src = ./.;
        pname = (pkgs.lib.importTOML ./Cargo.toml).package.name;
        buildInputs = darwin-frameworks;
      };

      apps.default = utils.lib.mkApp {
        drv = self.packages."${system}".default;
      };

      devShells.default = with pkgs;
        mkShell {
          packages = [toolchain taplo] ++ darwin-frameworks;
        };

      nixosModules.default = {config, ...}: let
        name = "undefspace-tg-bot";
      in
        with nixpkgs.lib; {
          options = {
            services.${name} = {
              enable = mkEnableOption "enables undefspace telegram bot";
              config = mkOption {
                type = types.path;
                default = null;
                description = ''
                  path to environment config
                '';
                example = ''
                  writeTextFile '''
                    TELOXIDE_TOKEN="telegram token"
                    HASS_TOKEN="home assistant token"
                    CONTROL_CHAT_ID="id of a control chat"
                  ''';
                '';
              };
            };
          };
          config = mkIf config.services.${name}.enable {
            systemd.services.${name} = {
              serviceConfig.Restart = "always";
              wantedBy = ["multi-user.target"];
              after = ["network.target"];
              script = ''
                . ${config.services.${name}.config}
                ${self.defaultPackage."${system}"}/bin/${name}
              '';
            };
          };
        };
    });
}
