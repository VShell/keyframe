{ lib, pkgs, config, ... }:
let
  cfg = config.keyframe;
  streamsFile = pkgs.writeText "streams" (lib.concatStringsSep "\n" (lib.mapAttrsToList (k: v: "${k}\t${v.email}\t${if v.jid != null then v.jid else ""}") cfg.streams));
  stream-muc-manager = pkgs.callPackage ./stream-muc-manager {};
in lib.mkIf cfg.enable {
  systemd.services.create-xmpp-user-stream-muc-manager = {
    description = "Create stream-muc-manager@streamadmin.${cfg.domain}";
    serviceConfig = {
      Type = "oneshot";
      StateDirectory = "/var/lib/keyframe/stream-muc-manager";
      ConditionPathExists = "!/var/lib/keyframe/stream-muc-manager/xmpp-password";
      IgnoreSIGPIPE = false;
    };
    script = ''
      password=$(tr -dc abcdefghijklmnopqrstuvwxyz < /dev/urandom | fold -w 20 | head -n1)
      install -m 600 <(printf '%s' $password) /var/lib/stream-muc-manager/xmpp-password
      ${pkgs.prosody}/bin/prosodyctl register stream-muc-manager streamadmin.${cfg.domain} $password
    '';
  };

  systemd.services.create-streams = {
    description = "Create/delete streams";
    wantedBy = [ "multi-user.target" ];
    before = [ "multi-user.target" ];
    requires = [ "prosody.service" "create-xmpp-user-stream-muc-manager.service" "acme-prosody-${cfg.domain}.service" ];
    after = [ "prosody.service" "create-xmpp-user-stream-muc-manager.service" "acme-prosody-${cfg.domain}.service" ];
    serviceConfig = {
      Type = "oneshot";
      IgnoreSIGPIPE = false;
    };
    script = ''
      mapfile -t streams <${streamsFile}
      declare -A emails
      declare -A jids
      for i in "''${!streams[@]}"; do
        stream=$(printf '%s' "''${streams[$i]}" | cut -f1)
        email=$(printf '%s' "''${streams[$i]}" | cut -f2)
        jid=$(printf '%s' "''${streams[$i]}" | cut -f3)
        streams[$i]=$stream
        emails[$stream]="$email"
        jids[$stream]="$jid"
      done

      tmp=$(mktemp -d)
      trap "{ rm -r $tmp; }" EXIT
      ensure=$tmp/ensure
      remove=$tmp/remove

      exec {ensurefd}<> >(exec install -m 600 /dev/stdin $ensure)
      exec {removefd}<> >(exec install -m 600 /dev/stdin $remove)
      exec {rtmpusersnewfd}<> >(exec install -o rtmpauth -g rtmpauth -m 660 /dev/stdin /var/lib/rtmpauth/users.new)

      genpassword() {
        tr -dc abcdefghijklmnopqrstuvwxyz < /dev/urandom | fold -w 20 | head -n1
      }

      # Make sure all these streams still have a room
      for stream in "''${streams[@]}"; do
        jid="''${jids[$stream]:-$stream@${cfg.domain}}"
        printf '%s\t%s\n' $stream@streamchat.${cfg.domain} $jid >&$ensurefd
      done
      exec {ensurefd}>&-

      [[ -d /var/lib/rtmpauth ]] || install -o rtmpauth -g rtmpauth -m 766 -d /var/lib/rtmpauth
      # Check which streams to keep
      while IFS= read -r line; do
        stream=$(printf '%s' $line | cut -d: -f1)
        for i in "''${!streams[@]}"; do
          if [[ $stream == ''${streams[$i]} ]]; then
            printf '%s\n' $line >&$rtmpusersnewfd
            unset streams[$i]
            continue 2
          fi
        done
        # Stream not in new array, remove its room
        printf '%s\n' $stream@streamchat.${cfg.domain} >&$removefd
      done < /var/lib/rtmpauth/users
      exec {removefd}>&-

      # Add new streams
      for stream in "''${streams[@]}"; do
        streamkey=$(genpassword)

        printf '%s:%s\n' $stream $streamkey >&$rtmpusersnewfd
        if [[ -z "''${jids[$stream]}" ]]; then
          password=$(genpassword)
          ${pkgs.prosody}/bin/prosodyctl register $stream ${cfg.domain} $password
        fi

        # Send an email with the relevant info
        email="''${emails[$stream]}"
        exec {emailfd}<> >(exec install -m 600 /dev/stdin $tmp/email)
        printf 'Subject: New stream at ${cfg.domain}\n\n' >&$emailfd
        printf 'Stream URL: https://${cfg.domain}/publish/%s\n' $stream >&$emailfd
        printf 'Stream RTMP URL: rtmp://${cfg.domain}/publish/%s\n' $streamkey >&$emailfd
        if [[ -n "''${jids[$stream]}" ]]; then
          printf 'XMPP username: %s@${cfg.domain}' $stream >&emailfd
          printf 'XMPP password: %s' $password >&emailfd
        fi
        exec {emailfd}>&-
        ${pkgs.system-sendmail}/bin/sendmail "$email" <$tmp/email || true
      done
      exec {rtmpusersnewfd}>&-
      mv /var/lib/rtmpauth/users{.new,}

      # Ask rtmpauth to reload its config, if it's up
      if systemctl is-active rtmpauth --quiet; then
        systemctl reload rtmpauth || true
      fi

      # Add/remove XMPP rooms
      password=$(</var/lib/keyframe/stream-muc-manager/xmpp-password)
      ${stream-muc-manager}/bin/stream-muc-manager -jid stream-muc-manager@streamadmin.${cfg.domain} -password "$password" -ensure $ensure -remove $remove
    '';
  };
}
