{ lib, pkgs, config, ... }:
with lib;
let
  cfg = config.keyframe;
in {
  options.keyframe = {
    enable = mkEnableOption "Keyframe video streaming server";
    domain = mkOption {
      type = types.str;
      description = ''
        The domain to run Keyframe on. This domain should not be used for anything else.
        Subdomains streamchat, streamguest and streamadmin must also exist under this
        domain.
      '';
    };
    streams = mkOption {
      type = types.attrsOf (types.submodule {
        options = {
          email = mkOption {
            type = types.str;
            description = ''
              Email address of the owner of this stream. The owner will receive an email
              with their stream key and XMPP password, if sendmail is configured on this
              host.
            '';
          };
          jid = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = ''
              JID of an existing XMPP account owned by the owner of this stream. This
              account will be given admin access to the stream's XMPP chatroom.
            '';
          };
        };
      });
      description = "Streams to create on this server."
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
