{ lib, config, pkgs, ... }:
let
  cfg = config.keyframe;
  ui = pkgs.callPackage ./ui {};
  acmeVhost = domain:
    lib.nameValuePair domain
    (lib.mkIf (cfg.tls.mode == "letsencrypt") {
      locations."/.well-known/acme-challenge".root = "/var/lib/acme/acme-challenge";
    });
  acmeVhosts = lib.listToAttrs (map acmeVhost [
    "streamguest.${cfg.domain}"
    "streamchat.${cfg.domain}"
    "streamadmin.${cfg.domain}"
  ]);
in lib.mkIf cfg.enable {
  services.nginx = {
    enable = true;
    appendHttpConfig = ''
      server_names_hash_bucket_size 128;
      proxy_http_version 1.1;
      proxy_set_header Host $host;
    '';
    virtualHosts = {
      "${cfg.domain}" = {
        enableACME = cfg.tls.mode == "letsencrypt";
        sslCertificate = lib.mkIf (cfg.tls.mode != "letsencrypt") cfg.tls.certificate;
        sslCertificateKey = lib.mkIf (cfg.tls.mode != "letsencrypt") cfg.tls.key;
        forceSSL = true;
        locations = {
          "= /" = {
            extraConfig = ''
              return 301 /stream/;
            '';
          };
          "/stream/" = {
            root = "${ui}";
            tryFiles = "/index.html =404";
          };
          "/view/" = {
            root = "${ui}";
            tryFiles = "/index.html =404";
          };
          "~ ^/stream/[a-zA-Z0-9]+.mpd$" = {
            proxyPass = "http://unix:/run/keyframe/streamredirect/http.sock";
          };
          "= /api/v1/ingestd-notify" = {
            proxyPass = "http://unix:/run/keyframe/streamredirect/http.sock";
          };
          "/stream-meta/ui/" = {
            root = "${ui}";
          };
          "/stream-meta/logos/" = {
            alias = "/var/lib/keyframe/logos/";
          };
          "= /stream-meta/bosh" = {
            proxyPass = "http://localhost:5280/http-bind";
          };
        };
      };
      "ingestd.${cfg.domain}" = {
        enableACME = cfg.tls.mode == "letsencrypt";
        sslCertificate = lib.mkIf (cfg.tls.mode != "letsencrypt") cfg.tls.certificate;
        sslCertificateKey = lib.mkIf (cfg.tls.mode != "letsencrypt") cfg.tls.key;
        forceSSL = true;
        extraConfig = ''
          proxy_buffering off;
        '';
        locations = {
          "/" = {
            proxyPass = "http://unix:/run/ingestd/httpd/http-public.sock";
          };
        };
      };
    } // acmeVhosts;
  };
}
