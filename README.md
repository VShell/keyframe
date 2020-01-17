# Keyframe

Keyframe is a self-hosted live video streaming server with integrated community features.

## Getting Started

On a [NixOS](https://nixos.org) server, add the following to /etc/nixos/keyframe.nix:

```
let
  keyframe = (builtins.fetchTarball https://keyframe.alterednarrative.net/download/0.1.tar.gz)
in
{
  imports = [
    keyframe
  ];

  keyframe = {
    enable = true;
    domain = "stream.yourdomain.example";
    streams = {
      test = { email = "your@email.example"; };
    };
  };
}
```

Then import it in your configuration.nix, by adding:

```
# ...
  imports = [
    # ...
    ./keyframe.nix
  ];
# ...
```

You should also set up a mailserver that can send email.
[simple-nixos-mailserver](https://gitlab.com/simple-nixos-mailserver/nixos-mailserver)
is a good choice if you don't have one already.

After this, run `sudo nixos-rebuild switch`, and check your email!

## License

Keyframe is released under various licenses. Some components are licensed under the AGPL
or the GPL, while the NixOS configuration is licensed under the MIT. See the
[LICENSE](LICENSE) file for more details.

This licensing choice ensures that Keyframe contributes to the free software commons.
Anyone can use, modify and redistribute the software, creating bespoke servers for their
own communities; in return, they must publish their modifications under the same terms.
