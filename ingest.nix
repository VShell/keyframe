{ pkgs, ... }:
{
  # todo: We should probably run our own ingest, really
  services.nginx = {
    enable = true;
    package = pkgs.nginxStable.override {
      modules = [
        pkgs.nginxModules.rtmp
      ];
    };
    appendConfig = ''
      rtmp {
        server {
          listen 1935;
          application publish {
            live on;

            deny play all;

            on_publish http://127.0.0.1:1337/on_publish;
            push rtmp://127.0.0.1:1935/stream;
          }
          application stream {
            live on;

            allow publish 127.0.0.1;
            deny publish all;

            hls on;
            hls_path /tmp/hls;
            hls_nested on;
            hls_fragment 3s;
          }
        }
      }
    '';
  };
}
