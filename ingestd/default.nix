{ rustPlatform, sqlite, pkgconfig, srt, llvmPackages, openssl }:

rustPlatform.buildRustPackage rec {
  pname = "ingestd";
  version = "0.1";
  src = ./.;
  cargoSha256 = "08fmqa6h98sr2ckyqm9j63slj00jir7a5wmwsi2chglyzdxxygpd";
  verifyCargoDeps = true;
  cargoBuildFlags = [ "-p" "ingestd-httpd" "-p" "ingestd-srt" ];
  nativeBuildInputs = [ pkgconfig llvmPackages.clang ];
  buildInputs = [ srt openssl ];
  LIBCLANG_PATH = "${llvmPackages.libclang}/lib";
  RUSTC_BOOTSTRAP = 1;
  preBuild = ''
    database_path=$(mktemp -d)/streams.db
    ${sqlite}/bin/sqlite3 $database_path < ${./.}/ingestd-srt/schema.sql
    export DATABASE_URL=sqlite:$database_path
  '';
}
