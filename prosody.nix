{ lib, config, ... }:
let
  cfg = config.keyframe;
  certPaths = if cfg.tls.mode == "letsencrypt" then
    {
      certificate = "/var/lib/acme/prosody-${cfg.domain}/fullchain.pem";
      key = "/var/lib/acme/prosody-${cfg.domain}/key.pem";
    }
  else
    {
      inherit (cfg.tls) certificate key;
    };
in lib.mkIf cfg.enable {
  security.acme.certs."prosody-${cfg.domain}" = lib.mkIf (cfg.tls.mode == "letsencrypt") {
    domain = cfg.domain;
    extraDomains = {
      "streamchat.${cfg.domain}" = null;
      "streamguest.${cfg.domain}" = null;
      "streamadmin.${cfg.domain}" = null;
     };
    webroot = "/var/lib/acme/acme-challenge";
    user = config.services.nginx.user;
    group = "prosody";
    allowKeysForGroup = true;
    postRun = ''
      systemctl reload prosody
    '';
  };

  services.prosody = {
    enable = true;
    xmppComplianceSuite = false;
    modules = {
      bosh = true;
    };
    extraConfig = ''
      consider_bosh_secure = true
    '';
    httpsPorts = [];
    virtualHosts = {
      "${cfg.domain}" = {
        enabled = true;
        domain = "${cfg.domain}";
        ssl = {
          cert = certPaths.certificate;
          key = certPaths.key;
        };
      };
      "streamguest.${cfg.domain}" = {
        enabled = true;
        domain = "streamguest.${cfg.domain}";
        ssl = {
          cert = certPaths.certificate;
          key = certPaths.key;
        };
        extraConfig = ''
          authentication = "anonymous"
        '';
      };
      "streamadmin.${cfg.domain}" = {
        enabled = true;
        domain = "streamadmin.${cfg.domain}";
        ssl = {
          cert = certPaths.certificate;
          key = certPaths.key;
        };
      };
    };
    muc = [ {
      domain = "streamchat.${cfg.domain}";
      name = "${cfg.domain} stream chatrooms";
      restrictRoomCreation = "admin";
      tombstones = false;
      extraConfig = ''
        admins = { "stream-muc-manager@streamadmin.${cfg.domain}" }
      '';
    } ];
  };

  systemd.services.prosody = lib.mkIf (cfg.tls.mode == "letsencrypt") {
    wants = [ "acme-prosody-${cfg.domain}.service" "acme-selfsigned-prosody-${cfg.domain}.service" ];
    after = [ "acme-selfsigned-prosody-${cfg.domain}.service" ];
  };
}
