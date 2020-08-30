{ lib, config, pkgs, ... }:
let
  cfg = config.keyframe;
  cms = pkgs.callPackage ./cms {};
  streamredirectConfig = {
    trusted-proxies = [ "127.0.0.1" ];
    database = "/var/lib/keyframe/streams.db";
  };
  streamredirectConfigFile = pkgs.runCommand "streamredirect.toml" {
    json = builtins.toJSON streamredirectConfig;
    passAsFile = [ "json" ];
  } "${pkgs.remarshal}/bin/json2toml < $jsonPath > $out";
in lib.mkIf cfg.enable {
  users.groups.keyframe = {};
  users.users.keyframe = {
    description = "Keyframe service user";
    group = "keyframe";
    isSystemUser = true;
    home = "/var/lib/keyframe";
  };

  systemd.services.streamredirect = {
    description = "Stream redirection server between Keyframe and ingestd";
    preStart = ''
      if [[ ! -e /var/lib/keyframe/streams.db ]]; then
        install -m 600 /dev/null /var/lib/keyframe/streams.db
        ${pkgs.sqlite}/bin/sqlite3 /var/lib/keyframe/streams.db < ${./cms/streamredirect/schema.sql}
      fi
    '';
    serviceConfig = {
      StateDirectory = "keyframe";
      ExecStart = "${cms}/bin/streamredirect ${streamredirectConfigFile}";
      User = "keyframe";
      StandardInput = "socket";
      StandardOutput = "journal";
    };
  };

  systemd.sockets.streamredirect = {
    description = "Stream redirection server between Keyframe and ingestd";
    wantedBy = [ "sockets.target" ];
    listenStreams = [ "/run/keyframe/streamredirect/http.sock" ];
    socketConfig = {
      RuntimeDirectory = "keyframe/streamredirect";
    };
  };
}
