{ lib, config, pkgs, ... }:
let
  cfg = config.keyframe;
  webapp = pkgs.callPackage ./webapp {};
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
    appendHttpConfig = ''
      server_names_hash_bucket_size 128;
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
            root = "${webapp}";
            tryFiles = "/index.html =404";
            extraConfig = ''
              rewrite ^/stream/(?<stream>[a-zA-Z0-9]+)\.m3u8$ /stream-meta/hls/$stream/ redirect;
            '';
          };
          "/stream-meta/webapp/" = {
            alias = "${webapp}/dist/";
          };
          "/stream-meta/hls/" = {
            alias = "/tmp/hls/";
            extraConfig = ''
              types {
                application/x-mpegURL m3u8;
                video/mp2t ts;
              }
              index index.m3u8;
              add_header Cache-Control no-cache;
              add_header Access-Control-Allow-Origin *;
            '';
          };
          "= /stream-meta/bosh" = {
            proxyPass = "http://localhost:5280/http-bind";
          };
        };
      };
    } // acmeVhosts;
  };
}
