{ pkgs ? import <nixpkgs> {} }:
let
  pkgPath = "${pkgs.glib.dev}/lib/pkgconfig:${pkgs.cairo.dev}/lib/pkgconfig:${pkgs.pango.dev}/lib/pkgconfig:${pkgs.harfbuzz.dev}/lib/pkgconfig";
in
pkgs.rustPlatform.buildRustPackage rec {
  pname = "penrose";
  version = "0.0.2";
  src = ./.;
  cargoSha256 = "03ab5lmbbccs34032fm4ql2394m301dr0k4myyrcmjn8m1093xg2";
  nativeBuildInputs = with pkgs; [
    glib.dev
    cairo.dev
    pango.dev
    harfbuzz.dev
    pkg-config
    python3
    xorg.libxcb.dev
    xorg.libXrandr.dev
    xorg.libXrender.dev
    xorg.xmodmap
  ];

#  buildPhase = ''
#    cargo build
#  '';
#  installPhase = ''
#    mkdir -p $out
#    cp -r target/debug $out/bin
#  '';

  buildInputs = nativeBuildInputs;
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeBuildInputs;
  PKG_CONFIG_PATH= pkgPath;
  doCheck = false;
}
