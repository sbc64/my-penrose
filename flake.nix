{
  /*
  shell
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
  */


  /*
  default
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
  */
  description = "Penrose Flake";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nmattia/naersk/master";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    flake-compat = {
      url = github:edolstra/flake-compat;
      flake = false;
    };
  };

  outputs = { self, nixpkgs, naersk }:
  let
    pkgs = import nixpkgs { };
    naersk-lib = pkgs.callPackage naersk { };
  in {
    defaultPackage.x86_64-linux =
      with import nixpkgs { system = "x86_64-linux"; };
      naersk-lib.buildPackage {
        src = ./.;
        buildInputs = with pkgs; [ pkg-config openssl ];
      };
    devShell.x86_64-linux = pkgs.mkShell {
      buildInputs = with pkgs; [
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
      RUST_LOG = "info";
      RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
    };
  };

}
