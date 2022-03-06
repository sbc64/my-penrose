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
   */
  description = "Penrose Flake";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk/master";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    naersk,
    utils,
    flake-compat,
  }: utils.lib.eachDefaultSystem (system: 
  let
    pkgs = nixpkgs.legacyPackages."${system}";
    naersk-lib = naersk.lib."${system}";
  in {
    defaultPackage = naersk-lib.buildPackage {
        pname = "my-penrose";
        root = ./.;
        buildInputs = with pkgs; [pkg-config openssl];
    };
    devShell = pkgs.mkShell {
      buildInputs = with pkgs; [
        cargo
        rustc
        clippy
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
  });
}
