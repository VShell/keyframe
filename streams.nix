{ lib, pkgs, config, ... }:
let
  cfg = config.keyframe;
  streamsFile = pkgs.writeText "streams.json" (builtins.toJSON cfg.streams);
  stream-muc-manager = pkgs.callPackage ./stream-muc-manager {};
  prosodyCertService = lib.optional (cfg.tls.mode == "letsencrypt") "acme-prosody-${cfg.domain}.service";
  python = pkgs.python3.withPackages (ps: [ps.toml]);
in lib.mkIf cfg.enable {
  systemd.services.create-xmpp-user-stream-muc-manager = {
    description = "Create stream-muc-manager@streamadmin.${cfg.domain}";
    serviceConfig = {
      Type = "oneshot";
      StateDirectory = "keyframe/stream-muc-manager";
      ConditionPathExists = "!/var/lib/keyframe/stream-muc-manager/xmpp-password";
      IgnoreSIGPIPE = false;
    };
    script = ''
      password=$(tr -dc abcdefghijklmnopqrstuvwxyz < /dev/urandom | fold -w 20 | head -n1)
      install -m 600 <(printf '%s' $password) /var/lib/keyframe/stream-muc-manager/xmpp-password
      ${pkgs.prosody}/bin/prosodyctl register stream-muc-manager streamadmin.${cfg.domain} $password
    '';
  };

  systemd.services.create-streams = {
    description = "Create/delete streams";
    wantedBy = [ "multi-user.target" ];
    before = [ "multi-user.target" ];
    requires = [ "prosody.service" "create-xmpp-user-stream-muc-manager.service" "ingestd-srt.service" "streamredirect.service" ] ++ prosodyCertService;
    after = [ "prosody.service" "create-xmpp-user-stream-muc-manager.service" "ingestd-srt.service" "streamredirect.service" ] ++ prosodyCertService;
    path = [ pkgs.prosody pkgs.b3sum pkgs.system-sendmail stream-muc-manager ];
    environment = {
      streamsPath = streamsFile;
      domain = cfg.domain;
    };
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${python}/bin/python3 ${./create_streams.py}";
    };
  };
}
