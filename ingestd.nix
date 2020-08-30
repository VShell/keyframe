{ lib, config, pkgs, ... }:
let
  cfg = config.keyframe;
  ingestd = pkgs.callPackage ./ingestd {};
  gpac = pkgs.callPackage ./gpac {};
  systemd-socket-activation-shim = pkgs.callPackage ./systemd-socket-activation-shim {};
  ingestd-srtConfig = {
    stream-logs = "/var/log/ingestd";
    httpd-url = "http://127.0.0.1:9000";
    external-url = "https://ingestd.${cfg.domain}";
    database = {
      url = "sqlite:/var/lib/ingestd/streams.db";
    };
  };
  ingestd-srtConfigFile = pkgs.writeText "ingestd-srt.json" (builtins.toJSON ingestd-srtConfig);
  ingestd-httpdConfig = {
    web-root = "/var/lib/ingestd/httpd/www";
  };
  ingestd-httpdConfigFile = pkgs.runCommand "ingestd-httpd.toml" {
    json = builtins.toJSON ingestd-httpdConfig;
    passAsFile = [ "json" ];
  } "${pkgs.remarshal}/bin/json2toml < $jsonPath > $out";
in lib.mkIf cfg.enable {
  users.groups.ingestd = {};
  users.users.ingestd = {
    description = "Ingestd service user";
    group = "ingestd";
    isSystemUser = true;
    home = "/var/lib/ingestd";
  };

  systemd.services.ingestd-srt = {
    description = "Ingestd SRT server";
    preStart = ''
      if [[ ! -e /var/lib/ingestd/streams.db ]]; then
        install -m 600 /dev/null /var/lib/ingestd/streams.db
        ${pkgs.sqlite}/bin/sqlite3 /var/lib/ingestd/streams.db < ${./ingestd/ingestd-srt/schema.sql}
      fi

      if [[ ! -e /var/lib/ingestd/ingestd-srt.toml ]]; then
        secret=\"$(dd status=none if=/dev/urandom bs=32 count=1 | base64 -)\"
      else
        secret=$(${pkgs.remarshal}/bin/toml2json /var/lib/ingestd/ingestd-srt.toml --unwrap secret)
      fi
      ${pkgs.jq}/bin/jq --argjson secret "$secret" '. + {secret: $secret}' ${ingestd-srtConfigFile} | ${pkgs.remarshal}/bin/json2toml -o /var/lib/ingestd/ingestd-srt.toml
    '';
    path = [ gpac ];
    serviceConfig = {
      StateDirectory = "ingestd";
      LogsDirectory = "ingestd";
      ExecStart = "${ingestd}/bin/ingestd-srt /var/lib/ingestd/ingestd-srt.toml";
      User = "ingestd";
      StandardInput = "socket";
      StandardOutput = "journal";
      ExecReload = "${pkgs.coreutils}/bin/kill -USR1 $MAINPID";
    };
  };

  systemd.sockets.ingestd-srt = {
    description = "Ingestd SRT server";
    wantedBy = [ "sockets.target" ];
    socketConfig = {
      ListenDatagram = "0.0.0.0:3800";
    };
  };

  systemd.services.ingestd-httpd = {
    description = "Ingestd HTTP server";
    preStart = ''
      [ -e /var/lib/ingestd/httpd/dumps ] || mkdir -p /var/lib/ingestd/httpd/dumps
    '';
    serviceConfig = {
      StateDirectory = "ingestd/httpd";
      ExecStart = "${systemd-socket-activation-shim}/bin/systemd-socket-activation-shim ingestd-httpd-public.socket '' '' ingestd-httpd-private.socket -- ${ingestd}/bin/ingestd-httpd ${ingestd-httpdConfigFile}";
      DynamicUser = true;
      Sockets = [ "ingestd-httpd-public.socket" "ingestd-httpd-private.socket" ];
      SyslogIdentifier = "ingestd-httpd";
    };
  };

  systemd.sockets.ingestd-httpd-public = {
    description = "Ingestd HTTP server";
    wantedBy = [ "sockets.target" ];
    listenStreams = [ "/run/ingestd/httpd/http-public.sock" ];
    socketConfig = {
      Service = "ingestd-httpd.service";
      RuntimeDirectory = "ingestd/httpd";
    };
  };

  systemd.sockets.ingestd-httpd-private = {
    description = "Ingestd HTTP server for internal use by ingestd-srt";
    wantedBy = [ "sockets.target" ];
    listenStreams = [ "127.0.0.1:9000" ];
    socketConfig = {
      Service = "ingestd-httpd.service";
    };
  };
}
