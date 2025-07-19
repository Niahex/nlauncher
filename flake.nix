{
  description = "nlauncher - A GTK-based application launcher for Wayland";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        # Overlays and package set
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};

        # Rust toolchain configuration
        rustToolchain = pkgs.rust-bin.stable."1.88.0".default.override {
          extensions = ["rust-src"];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        src = craneLib.cleanCargoSource ./.;

        # Common dependencies
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

        # Build artifacts
        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src buildInputs nativeBuildInputs;
          env = envVars;
        };

        # Application package definition
        nlauncher = craneLib.buildPackage {
          inherit src cargoArtifacts buildInputs nativeBuildInputs;
          env = envVars;
          pname = "nlauncher";
          version = "0.1.0";
        };

        # Development shell tools
        devTools = with pkgs; [
          rust-analyzer
          rustToolchain
          cargo-watch
          cargo-edit
          bacon
        ];
      in {
        packages = {
          default = nlauncher;
          inherit nlauncher;
        };

        checks = {
          inherit nlauncher;

          nlauncher-clippy = craneLib.cargoClippy {
            inherit src cargoArtifacts buildInputs nativeBuildInputs;
            env = envVars;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          };

          nlauncher-fmt = craneLib.cargoFmt {inherit src;};
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [nlauncher];
          nativeBuildInputs = devTools;
          env = envVars;

          shellHook = ''
            echo "[🦀 Rust $(rustc --version)] - Ready to develop nlauncher!"
          '';
        };

        formatter = pkgs.alejandra;
      }
    );
}
