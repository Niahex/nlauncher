{
  description = "nlauncher - A GPUI-based application launcher";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {self, nixpkgs, flake-utils, crane, rust-overlay, ...}:
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

        unfilteredRoot = ./.;
        src = pkgs.lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = pkgs.lib.fileset.unions [
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (pkgs.lib.fileset.fileFilter (
                file:
                  pkgs.lib.any file.hasExt [
                    "svg"
                  ]
              )
              unfilteredRoot)
            (pkgs.lib.fileset.maybeMissing ./assets)
          ];
        };

        # Dependencies for building the application
        buildInputs = with pkgs;
          [
            wayland
            vulkan-loader
            vulkan-validation-layers
            vulkan-tools
            mesa
            xorg.libxcb
            xorg.libX11
            libxkbcommon
            fontconfig
            dbus
            openssl
            freetype
            expat
            nerd-fonts.ubuntu-mono
            nerd-fonts.ubuntu-sans
            nerd-fonts.ubuntu
            noto-fonts-emoji
            libsecret
          ];

        # Dependencies needed only at runtime
        runtimeDependencies = with pkgs;
          [
            wayland
            vulkan-loader
            mesa
            libglvnd
            libxkbcommon
            wl-clipboard
            xorg.libX11
            xorg.libxcb
          ];

        nativeBuildInputs = with pkgs;
          [
            pkg-config
            makeWrapper
            autoPatchelfHook
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

          postInstall = ''
            wrapProgram $out/bin/nlauncher \
              --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath runtimeDependencies} \
              --suffix LD_LIBRARY_PATH : /run/opengl/driver/lib:/run/opengl/lib
          '';
        };

        # Development shell tools
        devTools = with pkgs;
          [
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

        homeManagerModules.default = import ./nix/home-manager.nix;

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

          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath (buildInputs ++ runtimeDependencies)}:/run/opengl/driver/lib:/run/opengl/lib";
          FONTCONFIG_FILE = pkgs.makeFontsConf {fontDirectories = buildInputs;};

          shellHook = ''
            echo "[🦀 Rust $(rustc --version)] - Ready to develop nlauncher!"
            echo "Vulkan ICD: $VK_ICD_FILENAMES"
            echo "Available Vulkan devices:"
            vulkaninfo --summary 2>/dev/null | grep -A 2 "GPU" || echo "  Run 'vulkaninfo' for details"
          '';
        };

        formatter = pkgs.alejandra;
      }
    );
}