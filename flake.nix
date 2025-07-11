{
  description = "nlauncher";

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

      toolchain = with fenix.packages.${system};
        combine [
          stable.toolchain
          targets.wasm32-unknown-unknown.stable.rust-std
        ];

      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

      src = craneLib.cleanCargoSource (craneLib.path ./.);

      nativeBuildInputs = with pkgs; [
        pkg-config
        cmake
      ];

      buildInputs = with pkgs; [
        libxkbcommon
        wayland
        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libXrandr
        xorg.libXext
        vulkan-loader
        vulkan-validation-layers
        libpulseaudio
        openssl
        freetype
        libGL
        libglvnd
        protobuf
        alsa-lib
      ];

      runtimeLibs = with pkgs; [
        libxkbcommon
        wayland
        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libXrandr
        vulkan-loader
      ];

      envVars = {
        RUST_BACKTRACE = "full";
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;
        XDG_RUNTIME_DIR = "/run/user/$(id -u)";
        WAYLAND_DISPLAY = "wayland-1";
        PROTOC = "${pkgs.protobuf}/bin/protoc";
      };

      cargoArtifacts = craneLib.buildDepsOnly {
        inherit src buildInputs nativeBuildInputs;
        env = envVars;
        doCheck = false;
      };

      nlauncher = craneLib.buildPackage {
        inherit src cargoArtifacts buildInputs nativeBuildInputs;
        env = envVars;
        pname = "nlauncher";
        version = "0.1.0";
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
          (fenix.packages.${system}.combine [
            fenix.packages.${system}.stable.toolchain
            fenix.packages.${system}.rust-analyzer
          ])
          cargo-watch
          cargo-edit
          pkg-config
          wayland-utils
          wl-clipboard
        ];

        buildInputs = runtimeLibs;

        shellHook = ''
          export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH
          export XDG_RUNTIME_DIR=/run/user/$(id -u)
          export WAYLAND_DISPLAY=wayland-1
          echo "🦀 Rust $(rustc --version) - egui ready"
          echo "🚀 Build with: nix build .#nlauncher"
        '';
      };

      formatter = pkgs.alejandra;
    });
}
