{ lib, config, pkgs, ... }:
let
  cfg = config.keyframe;
  webapp = pkgs.callPackage ./webapp {};
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
    virtualHosts."streamadmin.${cfg.domain}" = {
      locations = {
        "/.well-known/acme-challenge" = {
          root = "/var/lib/acme/acme-challenge";
        };
      };
    };
  };
}
