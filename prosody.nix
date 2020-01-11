{ config, ... }:
let
  cfg = config.video-streaming;
in
{
  security.acme.certs."prosody-${cfg.domain}" = {
    domain = cfg.domain;
    extraDomains = {
      "streamchat.${cfg.domain}" = null;
      "streamguest.${cfg.domain}" = null;
     };
    webroot = "/var/lib/acme/acme-challenge";
    group = "prosody";
    allowKeysForGroup = true;
    postRun = ''
      systemctl reload prosody
    '';
  };

  services.prosody = {
    enable = true;
    modules = {
      bosh = true;
      mam = true;
    };
    extraConfig = ''
      https_ports = { }
      consider_bosh_secure = true

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
        ssl = {
          cert = "/var/lib/acme/prosody-${cfg.domain}/fullchain.pem";
          key = "/var/lib/acme/prosody-${cfg.domain}/key.pem";
        };
      };
      "streamguest.${cfg.domain}" = {
        enabled = true;
        domain = "streamguest.${cfg.domain}";
        ssl = {
          cert = "/var/lib/acme/prosody-${cfg.domain}/fullchain.pem";
          key = "/var/lib/acme/prosody-${cfg.domain}/key.pem";
        };
        extraConfig = ''
          authentication = "anonymous"
        '';
      };
      "streamadmin.${cfg.domain}" = {
        enabled = true;
        domain = "streamadmin.${cfg.domain}";
        extraConfig = ''
          c2s_require_encryption = false
          allow_unencrypted_plain_auth = true
        '';
      };
    };
  };

  systemd.services.prosody = {
    wants = [ "acme-prosody-${cfg.domain}.service" "acme-selfsigned-prosody-${cfg.domain}.service" ];
    after = [ "acme-selfsigned-prosody-${cfg.domain}.service" ];
  };
}
