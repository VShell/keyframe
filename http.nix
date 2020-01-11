{ lib, config, ... }:
let
  cfg = config.video-streaming;
in lib.mkIf cfg.enable {
  services.nginx = {
    virtualHosts."${cfg.domain}" = {
      enableACME = true;
      forceSSL = true;
      locations = {
        "= /" = {
          extraConfig = ''
            return 301 /stream/;
          '';
        };
        "~ /stream/[a-zA-Z0-9]+$" = {
          alias = ./stream.html;
          extraConfig = ''
            types { }
            default_type text/html;
          '';
        };
        "~ /stream/(?<stream>[a-zA-Z0-9]+)\.m3u8$" = {
          extraConfig = ''
            return 302 /stream-meta/hls/$stream/index.m3u8;
          '';
        };
        "/stream-meta/hls/" = {
          alias = "/tmp/hls/";
          extraConfig = ''
            types {
              application/x-mpegURL m3u8;
              video/mp2t ts;
            }
            add_header Cache-Control no-cache;
            add_header Access-Control-Allow-Origin *;
          '';
        };
        "= /stream-meta/bosh" = {
          proxyPass = "http://localhost:5280/http-bind";
        };
      };
    };
    virtualHosts."streamguest.${cfg.domain}" = {
      locations = {
        "/.well-known/acme-challenge" = {
          root = "/var/lib/acme/acme-challenge";
        };
      };
    };
    virtualHosts."streamchat.${cfg.domain}" = {
      locations = {
        "/.well-known/acme-challenge" = {
          root = "/var/lib/acme/acme-challenge";
        };
      };
    };
  };
}
