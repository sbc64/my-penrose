{
  description = "My Rust project";

  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-20.03;
    import-cargo.url = github:edolstra/import-cargo;
  };

  outputs = { self, nixpkgs, import-cargo }: let

    inherit (import-cargo.builders) importCargo;

  in {

    defaultPackage.x86_64-linux =
      with import nixpkgs { system = "x86_64-linux"; };
      stdenv.mkDerivation {
        name = "penrose";
        src = self;

        nativeBuildInputs = [
          # setupHook which makes sure that a CARGO_HOME with vendored dependencies
          # exists
          (importCargo { lockFile = ./Cargo.lock; inherit pkgs; }).cargoHome

          # Build-time dependencies
          rustc cargo
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

        buildPhase = ''
          cargo build --release --offline
        '';

        installPhase = ''
          install -Dm775 ./target/release/testrust $out/bin/testrust
        '';

      };

  };
}
