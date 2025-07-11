{
  description = "nwidgets - A Ribir-based Wayland application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    crane,
    fenix,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      # Fenix est plus moderne et léger que rust-overlay
      toolchain = fenix.packages.${system}.stable.toolchain;

      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
      src = craneLib.cleanCargoSource ./.;

      # Dépendances runtime
      buildInputs = with pkgs; [
        gtk4
        gtk4-layer-shell
        glib
        pango
        gdk-pixbuf
        graphene
        libadwaita
        pkg-config
        wayland
        wayland-protocols
        dbus
        systemd
        networkmanager
        libnotify
        nerd-fonts.ubuntu-mono
        nerd-fonts.ubuntu-sans
        nerd-fonts.ubuntu
        openssl # Added for reqwest/openssl-sys compatibility
      ];

      # Dépendances build-time
      nativeBuildInputs = with pkgs; [
        pkg-config
        makeWrapper
        wrapGAppsHook4
        # Ajout de clang et libclang pour bindgen
        clang
        llvmPackages.libclang
      ];

      # Variables d'environnement
      envVars = {
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        RUST_BACKTRACE = "full";
      };

      # Artefacts cargo partagés pour optimiser les rebuilds
      cargoArtifacts = craneLib.buildDepsOnly {
        inherit src buildInputs nativeBuildInputs;
        env = envVars;
      };

      # Package principal
      nwidgets = craneLib.buildPackage {
        inherit src cargoArtifacts buildInputs nativeBuildInputs;
        env = envVars;
        pname = "nwidgets";
        version = "0.1.0";
        postInstall = ''
          install -Dm644 data/github.niahex.nwidgets.gschema.xml $out/share/glib-2.0/schemas/github.niahex.nwidgets.gschema.xml
        '';
      };
    in {
      # Output moderne avec checks
      packages = {
        default = nwidgets;
        nwidgets = nwidgets;
      };

      # Checks pour CI/CD
      checks = {
        inherit nwidgets;

        # Vérifications supplémentaires
        nwidgets-clippy = craneLib.cargoClippy {
          inherit src cargoArtifacts buildInputs nativeBuildInputs;
          env = envVars;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        };

        nwidgets-fmt = craneLib.cargoFmt {
          inherit src;
        };
      };

      # Dev shell moderne
      devShells.default = pkgs.mkShell {
        inputsFrom = [nwidgets];
        nativeBuildInputs = with pkgs; [
          # Outils de développement modernes
          fenix.packages.${system}.rust-analyzer
          fenix.packages.${system}.stable.toolchain

          # Outils additionnels
          cargo-watch
          cargo-edit
          bacon # cargo check continu
          # gemini-cli
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
