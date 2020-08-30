{ buildGoModule, sqlite }:
buildGoModule {
  name = "cms";
  src = ./.;
  vendorSha256 = "1q6bx7qaz3d8pwpcc38zyy56lnffvzbqlj6nzbdngqgv108fvdb6";
  subPackages = [ "streamredirect" ];
  CGO_ENABLED = 1;
  buildInputs = [ sqlite ];
  buildFlags = "-tags libsqlite3";
}
