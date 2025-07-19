{
  description = "nlauncher - A GTK-based application launcher for Wayland";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" ];
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
      src = craneLib.cleanCargoSource ./.;

      buildInputs = with pkgs; [
        gtk4
        gtk4-layer-shell
        glib
        pango
        gdk-pixbuf
        wayland
        wayland-protocols
        dbus
      ];

      nativeBuildInputs = with pkgs; [
        pkg-config
        makeWrapper
        wrapGAppsHook4
      ];

      envVars = {
        RUST_BACKTRACE = "full";
      };

      cargoArtifacts = craneLib.buildDepsOnly {
        inherit src buildInputs nativeBuildInputs;
        env = envVars;
      };

      nlauncher = craneLib.buildPackage {
        inherit src cargoArtifacts buildInputs nativeBuildInputs;
        env = envVars;
        pname = "nlauncher";
        version = "0.1.0";
        postInstall = ''
          install -d $out/share/glib-2.0/schemas
          cat > $out/share/glib-2.0/schemas/github.niahex.nlauncher.gschema.xml << EOF
          <?xml version="1.0" encoding="UTF-8"?>
          <schemalist>
            <schema id="github.niahex.nlauncher" path="/github/niahex/nlauncher/">
              <!-- Aucune clé de schéma pour le moment -->
            </schema>
          </schemalist>
          EOF
        '';
      };
    in {
      packages = {
        default = nlauncher;
        nlauncher = nlauncher;
      };

      checks = {
        inherit nlauncher;

        nlauncher-clippy = craneLib.cargoClippy {
          inherit src cargoArtifacts buildInputs nativeBuildInputs;
          env = envVars;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        };

        nlauncher-fmt = craneLib.cargoFmt {
          inherit src;
        };
      };

      devShells.default = pkgs.mkShell {
        inputsFrom = [nlauncher];
        nativeBuildInputs = with pkgs; [
          rust-analyzer
          rustToolchain
          cargo-watch
          cargo-edit
          bacon
        ];

        env = envVars;

        shellHook = ''
          echo "[🦀 Rust $(rustc --version)] - Ready !"
          echo "Dépendances: ${pkgs.lib.concatStringsSep " " (map (p: p.name) nativeBuildInputs)}"
          echo "Available commands: cargo watch, cargo edit, bacon"
        '';
      };

      formatter = pkgs.alejandra;
    });
}