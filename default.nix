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
    tls = mkOption {
      type = types.submodule {
        options = {
          mode = mkOption {
            type = types.enum [ "letsencrypt" "localcert" ];
            default = "letsencrypt";
            description = "Whether to fetch certificates from Let's Encrypt or use a local certificate";
          };
          certificate = mkOption {
            type = types.path;
            description = "Path to TLS certificate in localcert mode";
          };
          key = mkOption {
            type = types.path;
            description = "Path to TLS key in localcert mode";
          };
        };
      };
      default = {};
      description = "TLS configuration";
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
      description = "Streams to create on this server.";
    };
  };

  imports = [
    ./cms.nix
    ./ingestd.nix
    ./prosody.nix
    ./streams.nix
    ./http.nix
  ];
}
