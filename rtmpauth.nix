{ lib, config, pkgs, ... }:
let
  cfg = config.video-streaming;
  rtmpauth = pkgs.callPackage ./rtmpauth {};
in lib.mkIf cfg.enable {
  users.groups.rtmpauth = {};
  users.users.rtmpauth = {
    description = "rtmpauth service user";
    group = "rtmpauth";
    home = "/var/lib/rtmpauth";
  };

  systemd.services.rtmpauth = {
    description = "Authentication server for RTMP streaming";
    wantedBy = [ "multi-user.target" ];
    after = [ "networking.target" ];
    preStart = ''
      if [[ ! -e /var/lib/rtmpauth/users ]]; then
        install -m 600 /dev/null /var/lib/rtmpauth/users
      fi
    '';
    serviceConfig = {
      StateDirectory = "rtmpauth";
      ExecStart = "${rtmpauth}/bin/rtmpauth";
      ExecReload = "${pkgs.coreutils}/bin/kill -SIGUSR1 $MAINPID";
      User = "rtmpauth";
    };
  };
}
