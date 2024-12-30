{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  } @ inputs: let
      fenix = inputs.fenix.packages;
    in
    # Iterate over Arm, x86 for MacOs 🍎 and Linux 🐧
    (flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        crane = inputs.crane.mkLib pkgs;
        # Toolchain
        toolchain = fenix.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-s1RPtyvDGJaX/BisLT+ifVfuhDT1nZkZ1NcK8sbwELM=";
        };
        craneLib = crane.overrideToolchain toolchain;

        buildInputs = with pkgs; [
          openssl.dev
          pkg-config
          wayland
        ];

        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = path: type: (craneLib.filterCargoSources path type);
        };
        commonArgs = {
          doCheck = false;
          inherit src buildInputs;
        };

        libraries = with pkgs; [
          libxkbcommon
          wayland
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
        # Compile all artifacts
        appDeps = craneLib.buildDepsOnly commonArgs;

        # Compile
        pkg = name: craneLib.buildPackage (commonArgs // {
          cargoArtifacts = appDeps;
          cargoExtraArgs = "--example ${name}";
        });

        app = name: flake-utils.lib.mkApp {
          drv = pkg name;
        };
      in {
        # nix build
        packages.default = pkg "simple";

        # nix run
        apps.default = app "simple";

        # nix develop
        devShells.default = craneLib.devShell {
          inherit buildInputs;

          packages = [
            toolchain
          ] ++ libraries;
        };
      }
    ));
}
