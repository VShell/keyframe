{ lib, pkgs, config, ... }:
with lib;
let
  cfg = config.keyframe;
in {
  options.keyframe = {
    enable = mkEnableOption "Keyframe video streaming server";
    domain = mkOption {
      type = types.str;
    };
    streams = mkOption {
      type = types.attrsOf (types.submodule {
        options = {
          email = mkOption {
            type = types.str;
          };
          jid = mkOption {
            type = types.nullOr types.str;
            default = null;
          };
        };
      });
    };
  };

  imports = [
    ./rtmpauth.nix
    ./ingest.nix
    ./prosody.nix
    ./streams.nix
    ./http.nix
  ];

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [
      (pkgs.substituteAll {
        src = ./genstreamkey.sh;
        name = "genstreamkey";
        dir = "bin";
        isExecutable = true;
        bash = "${pkgs.bash}/bin/bash";
      })
    ];
  };
}
