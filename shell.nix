{ pkgs ? import <nixpkgs> {} }:
let
  pkgPath = "${pkgs.glib.dev}/lib/pkgconfig:${pkgs.cairo.dev}/lib/pkgconfig:${pkgs.pango.dev}/lib/pkgconfig:${pkgs.harfbuzz.dev}/lib/pkgconfig";
  toInstall = with pkgs; [
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
in
pkgs.mkShell {
  name = "penrose";
  buildIputs =  toInstall;
  nativeBuildInputs = toInstall;
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath toInstall;
  PKG_CONFIG_PATH= pkgPath;
}
