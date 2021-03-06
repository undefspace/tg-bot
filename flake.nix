{

  inputs = {
    naersk.url = "github:nix-community/naersk/master";
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

        defaultApp = utils.lib.mkApp {
            drv = self.defaultPackage."${system}";
        };

        devShell = with pkgs; mkShell {
          buildInputs = [ cargo rustc rustfmt pre-commit rustPackages.clippy openssl pkg-config ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };

        nixosModule = { config, ... }:
        let name = "undefspace-tg-bot";
        in with nixpkgs.lib; {
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
                    wantedBy = [ "multi-user.target" ];
                    after = [ "network.target" ];
                    script = ''
                      . ${config.services.${name}.config}
                      ${self.defaultPackage."${system}"}/bin/${name}
                    '';
                };

            };
        };

      });

}
