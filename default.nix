{ pkgs ? import <nixpkgs> {} }:
let
  pkgPath = "${pkgs.glib.dev}/lib/pkgconfig:${pkgs.cairo.dev}/lib/pkgconfig:${pkgs.pango.dev}/lib/pkgconfig:${pkgs.harfbuzz.dev}/lib/pkgconfig";
in
pkgs.rustPlatform.buildRustPackage rec {
  pname = "penrose";
  version = "0.1.0";
  src = ./.;
  cargoSha256 = "xXwAK/ZwgPgjKIuYvXa5pUVqUsTv186IlkK7SSYNd3c=";
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
  buildInputs = nativeBuildInputs;
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeBuildInputs;
  PKG_CONFIG_PATH= pkgPath;
  doCheck = false;
}
