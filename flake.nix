{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    utils.url = "github:numtide/flake-utils";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    crane,
    utils,
    pre-commit-hooks,
  }:
    utils.lib.eachDefaultSystem (system: let
      inherit (nixpkgs) lib;
      nixlib = lib;
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {inherit system overlays;};

      # General package name/version for the entire workspace.
      workspaceName = {
        pname = "beacon";
        # Use a dummy version.
        version = "0.0.0";
      };

      # Use crane to build rust packages.
      # Also sets the project specific rust toolchain in crane.
      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
      # Get the latest stable rust toolchain.
      toolchain = pkgs.rust-bin.stable.latest.default.override {
        # We need wasm32 in the toolchain to build the frontend.
        targets = ["wasm32-unknown-unknown"];
      };

      # Filtered source used for packages.
      # Reduces the amount of rebuilds needed, as it tries to detect
      # changes in any of the resulting files.
      #
      # Keeps:
      # - All rust related files
      # - All `.js` files
      # - All `.css` and `.scss` files
      # - `panel/assets`
      src = let
        filter = path: type:
          (craneLib.filterCargoSources path type)
          || (builtins.match ".*\\.js$" path != null)
          || (builtins.match ".*\\.s?css$" path != null)
          || (builtins.match "panel/assets.*" path != null);
      in
        lib.cleanSourceWith {
          src = craneLib.path ./.;
          inherit filter;
        };

      # Pre-build all native workspace dependencies.
      workspaceNativeDeps = craneLib.buildDepsOnly {
        inherit src;
        inherit (workspaceName) pname version;
      };

      # Local programs.
      programs = {
        cli = let
          inherit
            (craneLib.crateNameFromCargoToml {cargoToml = ./cli/Cargo.toml;})
            pname
            version
            ;
        in
          craneLib.buildPackage {
            inherit src pname version;

            cargoArtifacts = workspaceNativeDeps;
            cargoExtraArgs = "-p ${pname}";

            doCheck = false;
          };
      };

      # First make a non wrapped panel binary by compiling the rust project.
      # We wrap it later to allow it to always find the site root.
      panel-unwrapped = let
        pname = "beacon-panel";
        inherit
          (craneLib.crateNameFromCargoToml {cargoToml = ./panel/shared/Cargo.toml;})
          version
          ;

        panel-server = craneLib.crateNameFromCargoToml {cargoToml = ./panel/server/Cargo.toml;};
      in
        # The panel is a bit of a special case: it uses the `cargo leptos` build tool
        # along with both native and WASM dependencies.
        craneLib.buildPackage {
          inherit src pname version;

          cargoArtifacts = workspaceNativeDeps;
          nativeBuildInputs = with pkgs; [
            # Used to optimise WASM.
            binaryen
            # Build tool for SSR leptos applications.
            cargo-leptos
            # Enable tailwind support in the project.
            tailwindcss
            pkg-config
          ];
          # By default, `--locked` would be appended, which does not work with cargo-leptos.
          cargoExtraArgs = "";
          buildPhaseCargoCommand = "cargo leptos build --release -vvv";
          cargoTestCommand = "cargo leptos test --release -vvv";
          installPhaseCommand = ''
            mkdir -p $out/bin
            cp target/release/${panel-server.pname} $out/bin/panel
            mkdir -p $out/lib
            cp -r target/site $out/lib/
          '';

          # Extra check is not needed as we have a separate `cargo nextest` check.
          doCheck = false;
        };

      # Services for the file share service.
      # Each of these will also be used to create a docker image output.
      services = {
        # We wrap the panel in a shell script that sets the site root to the absolute nix store
        # path. This way we are sure the binary can always find the site root.
        panel = pkgs.writeShellScriptBin "panel" ''
          LEPTOS_SITE_ROOT=${panel-unwrapped}/lib/site ${panel-unwrapped}/bin/panel
        '';
      };

      # Generate a container image for each of the services.
      containers =
        lib.attrsets.concatMapAttrs (name: pkg: {
          "${name}" = pkgs.dockerTools.buildImage {
            inherit name;
            copyToRoot = pkg;
            tag = "latest";

            config = {
              Cmd = [name];
              WorkingDir = "/bin/site";
            };
          };
        })
        services;
    in rec {
      formatter = pkgs.alejandra;

      # The packages output by this flake is the combination of all programs,
      # all services and a container for each service (service name prefixed with '-container').
      packages =
        (nixlib.attrsets.concatMapAttrs (name: val: {"${name}-container" = val;}) containers)
        // services
        // programs;

      checks =
        {
          # Checks that are run when trying to commit. If one or more checks fail,
          # the commit is aborted.
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              # Github actions hooks.
              actionlint.enable = true;
              # Nix hooks.
              alejandra.enable = true;
              statix.enable = true;
              # Rust hooks.
              rustfmt.enable = true;
            };
          };

          # Run clippy.
          cargo-clippy = craneLib.cargoClippy {
            inherit src;
            inherit (workspaceName) pname version;
            cargoArtifacts = workspaceNativeDeps;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          };

          # Check cargo docs.
          cargo-doc = craneLib.cargoDoc {
            inherit src;
            inherit (workspaceName) pname version;
            cargoArtifacts = workspaceNativeDeps;
          };

          # Check formatting
          cargo-fmt = craneLib.cargoFmt {
            inherit src;
            inherit (workspaceName) pname version;
          };

          # Run tests with cargo-nextest
          cargo-nextest = craneLib.cargoNextest {
            inherit src;
            inherit (workspaceName) pname version;
            cargoArtifacts = workspaceNativeDeps;
            partitions = 1;
            partitionType = "count";
          };
        }
        // services
        // programs;

      devShells.default = with pkgs;
        pkgs.mkShell {
          inherit (self.checks.${system}.pre-commit-check) shellHook;

          # Take the inputs from the packages.
          inputsFrom =
            (builtins.attrValues packages)
            ++ [panel-unwrapped];

          packages = [
            # Add rust toolchain and make sure it has rust-analyzer.
            (toolchain.override {
              extensions = ["rust-analyzer" "rust-src"];
            })
            # Language Servers.
            vscode-langservers-extracted
            # Rust test runner (based on `cargo test`).
            cargo-nextest
            # Docker image inspector.
            dive
            # Script to automatically inspect the container image specified.
            #
            # Taken from: https://fasterthanli.me/series/building-a-rust-service-with-nix/part-11
            (pkgs.writeShellScriptBin "inspect-container" ''
              ${gzip}/bin/gunzip --stdout $1 > /tmp/image.tar && ${dive}/bin/dive docker-archive:///tmp/image.tar
            '')
          ];
        };

      lib = {
        # Matrix used for CI release pipeline.
        matrix.include =
          builtins.map
          (container: {inherit container;})
          (builtins.attrNames containers);
      };
    });
}
