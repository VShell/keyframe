{ lib, config, pkgs, ... }:
let
  cfg = config.keyframe;
  rtmpauth = pkgs.callPackage ./rtmpauth {};
in lib.mkIf cfg.enable {
  users.groups.rtmpauth = {};
  users.users.rtmpauth = {
    description = "rtmpauth service user";
    group = "rtmpauth";
    isSystemUser = true;
    home = "/var/lib/keyframe/rtmpauth";
  };

  systemd.services.rtmpauth = {
    description = "Authentication server for RTMP streaming";
    wantedBy = [ "multi-user.target" ];
    after = [ "networking.target" ];
    preStart = ''
      if [[ ! -e /var/lib/keyframe/rtmpauth/users ]]; then
        install -m 600 /dev/null /var/lib/keyframe/rtmpauth/users
      fi
    '';
    serviceConfig = {
      StateDirectory = "keyframe/rtmpauth";
      ExecStart = "${rtmpauth}/bin/rtmpauth";
      ExecReload = "${pkgs.coreutils}/bin/kill -SIGUSR1 $MAINPID";
      User = "rtmpauth";
    };
  };
}
