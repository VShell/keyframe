{ config, ... }:
let
  cfg = config.video-streaming;
in
{
  services.prosody = {
    enable = true;
    modules = {
      bosh = true;
      mam = true;
    };
    extraConfig = ''
      https_ports = { }
      consider_bosh_secure = true
      c2s_require_encryption = false
      allow_unencrypted_plain_auth = true

      Component "streamchat.${cfg.domain}" "muc"
        name = "${cfg.domain} stream chatrooms"
        restrict_room_creation = "admin"
        muc_tombstones = false
        admins = { "stream-muc-manager@streamadmin.${cfg.domain}" }
    '';
    virtualHosts = {
      "${cfg.domain}" = {
        enabled = true;
        domain = "${cfg.domain}";
      };
      "streamguest.${cfg.domain}" = {
        enabled = true;
        domain = "streamguest.${cfg.domain}";
        extraConfig = ''
          authentication = "anonymous"
        '';
      };
      "streamadmin.${cfg.domain}" = {
        enabled = true;
        domain = "streamadmin.${cfg.domain}";
      };
    };
  };
}
