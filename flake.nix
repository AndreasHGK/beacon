{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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

  outputs = { self, nixpkgs, rust-overlay, crane, utils, pre-commit-hooks, }:
    utils.lib.eachDefaultSystem (system:
      let
        inherit (nixpkgs) lib;
        nixlib = lib;
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

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
          targets = [ "wasm32-unknown-unknown" ];
        };

        # See `sources.nix` for more info.
        sources = import ./sources.nix { inherit lib craneLib; };

        # Pre-build dependencies for packages so they can be reused.
        dependencies = {
          rust = craneLib.buildDepsOnly {
            src = sources.rust;
            inherit (workspaceName) pname version;

            nativeBuildInputs = with pkgs;
              [
                # Used to find dependencies while building.
                pkg-config
                # SSL support.
                openssl
              ] ++ lib.optionals pkgs.stdenv.isDarwin [
                darwin.apple_sdk.frameworks.AppKit
                darwin.apple_sdk.frameworks.CoreGraphics
                darwin.apple_sdk.frameworks.SystemConfiguration
              ];
          };
        };

        # Local programs.
        programs = {
          cli = let
            inherit (craneLib.crateNameFromCargoToml {
              cargoToml = ./cli/Cargo.toml;
            })
              pname version;
          in craneLib.buildPackage {
            src = sources.rust;
            inherit pname version;

            cargoArtifacts = dependencies.rust;
            cargoExtraArgs = "-p ${pname}";

            inherit (dependencies.rust) nativeBuildInputs buildInputs;

            doCheck = false;
          };
        };

        # Services for the file share service.
        # Each of these will also be used to create a docker image output.
        services = {
          server = let
            inherit (craneLib.crateNameFromCargoToml {
              cargoToml = ./server/Cargo.toml;
            })
              pname version;
          in craneLib.buildPackage {
            src = sources.rust;
            inherit pname version;

            cargoArtifacts = dependencies.rust;
            cargoExtraArgs = "-p ${pname}";

            inherit (dependencies.rust) nativeBuildInputs buildInputs;

            doCheck = false;
          };

          frontend = pkgs.mkYarnPackage {
            pname = "beacon-frontend";
            src = sources.frontend;

            nativeBuildInputs = with pkgs; [ pkg-config yarn ];
            buildInputs = with pkgs; [ bash nodejs-slim_21 ];

            configurePhase = ''
              # We can't link as this would for some reason also link the full node modules in
              # the derivation's output, wasting a significant amount of space in the final
              # container.
              cp -r $node_modules node_modules
            '';
            buildPhase = ''
              export HOME=$(mktemp -d)
              yarn --offline run build
            '';
            distPhase = "true";
            installPhase = ''
              mkdir -p $out
              cp -R .next/standalone/. $out
              mkdir -p $out/.next/static
              cp -R .next/static/. $out/.next/static
              cp -R public $out/public

              # Create a binary to be able to easily launch the frontend.
              mkdir -p $out/bin
              echo "#! ${pkgs.bash}/bin/bash" >> $out/bin/frontend
              echo "${pkgs.nodejs-slim_20}/bin/node $out/server.js" >> $out/bin/frontend
              chmod +x $out/bin/frontend
            '';
          };
        };

        # Generate a container image for each of the services.
        containers = lib.attrsets.concatMapAttrs (name: pkg: {
          "${name}" = pkgs.dockerTools.buildImage {
            inherit name;
            copyToRoot = pkg;
            tag = "latest";

            config = {
              Cmd = [ name ];
              WorkingDir = "/";
            };
          };
        }) services;
      in rec {
        formatter = pkgs.nixfmt;

        # The packages output by this flake is the combination of all programs,
        # all services and a container for each service (service name prefixed with '-container').
        packages = (nixlib.attrsets.concatMapAttrs
          (name: val: { "${name}-container" = val; }) containers) // services
          // programs;

        checks = let
          targets = [ dependencies.rust ] ++ (builtins.attrValues services)
            ++ (builtins.attrValues programs);
          nativeBuildInputs = builtins.map (v: v.nativeBuildInputs) targets;
          buildInputs = builtins.map (v: v.buildInputs) targets;
        in {
          # Checks that are run when trying to commit. If one or more checks fail,
          # the commit is aborted.
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              # Github actions hooks.
              actionlint.enable = true;
              # Nix hooks.
              nixfmt.enable = true;
              statix.enable = true;
              # Rust hooks.
              rustfmt.enable = true;
              # JS hooks.
              prettier = {
                enable = true;
                settings = {
                  check = false;
                  list-different = true;
                  write = false;
                  configPath = "frontend/.prettierrc.json";
                };
              };
            };
          };

          # Run clippy.
          cargo-clippy = craneLib.cargoClippy {
            src = sources.rust;
            inherit (workspaceName) pname version;
            cargoArtifacts = dependencies.rust;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";

            inherit nativeBuildInputs buildInputs;
          };

          # Check cargo docs.
          cargo-doc = craneLib.cargoDoc {
            src = sources.rust;
            inherit (workspaceName) pname version;
            cargoArtifacts = dependencies.rust;

            inherit nativeBuildInputs buildInputs;

            # We only need the checks, not the artifacts. This significantly reduces the time the
            # fixupPhase takes.
            installPhase = ''
              mkdir -p $out
            '';
          };

          # Check formatting
          cargo-fmt = craneLib.cargoFmt {
            src = sources.rust;
            inherit (workspaceName) pname version;

            inherit nativeBuildInputs buildInputs;
          };

          # Run tests with cargo-nextest
          cargo-test = craneLib.cargoTest {
            src = sources.rust;
            inherit (workspaceName) pname version;
            cargoArtifacts = dependencies.rust;

            inherit nativeBuildInputs buildInputs;
          };

          # Ensure the frontend remains formatted.
          frontend-fmt = lib.mkCheck {
            name = "frontend-fmt";
            shellScript = ''
              cd ${sources.frontend}

              ${pkgs.nodePackages.prettier}/bin/prettier -c --config .prettierrc.json .
            '';
          };
        } // services // programs;

        devShells.default = with pkgs;
          pkgs.mkShell (let
            # Include rust analyzer into the dev toolchain.
            devToolchain = toolchain.override {
              extensions = [ "rust-analyzer" "rust-src" ];
            };
          in {
            inherit (self.checks.${system}.pre-commit-check) shellHook;

            # Take the inputs from the packages.
            inputsFrom = builtins.attrValues packages;

            packages = [
              # Add development rust toolchain.
              devToolchain
              # Docker image inspector.
              dive
              # JS/TS language server.
              nodePackages.typescript-language-server
              # NodeJS.
              nodejs_21
              # CLI for the sqlx library and database migrations.
              sqlx-cli
              # Language Servers.
              vscode-langservers-extracted
              # Frontend package manager.
              yarn
              # Script to automatically inspect the container image specified.
              #
              # Taken from: https://fasterthanli.me/series/building-a-rust-service-with-nix/part-11
              (pkgs.writeShellScriptBin "inspect-container" ''
                ${gzip}/bin/gunzip --stdout $1 > /tmp/image.tar && ${dive}/bin/dive docker-archive:///tmp/image.tar
              '')
            ];

            RUST_SRC_PATH = "${devToolchain}/lib/rustlib/src/rust/library";
          });

        apps = let
          # Helper function that turns a set with paths of derivations into a set of apps.
          mapToApp = builtins.mapAttrs (name: app: {
            program = "${app}/bin/${name}";
            type = "app";
          });
        in mapToApp {
          # Format the entire codebase.
          format = pkgs.writeShellScriptBin "format" ''
            echo "Formatting rust code ..."
            ${toolchain}/bin/cargo fmt
            echo "Formatting javascript code ..."
            ${pkgs.nodePackages.prettier}/bin/prettier -w --config ${
              ./.
            }/frontend/.prettierrc.json .
          '';
          # Run services used in development.
          dev = pkgs.writeShellApplication {
            name = "dev";

            runtimeInputs = with pkgs; [
              caddy
              cargo-watch
              nodePackages.yarn
              nss
              openssl
              toolchain
            ];

            text = ''
              # Kill all subprocesses on exit.
              trap 'trap - SIGTERM && kill -- -$$' SIGINT SIGTERM EXIT
              (cargo-watch -- cargo run --bin beacon-server) &
              (cd frontend && yarn run dev &)
              caddy run --envfile .env --config ./nix/Caddyfile
            '';
          };
        };

        lib = {
          # Matrix used for CI release pipeline.
          matrix.include = builtins.map (container: { inherit container; })
            (builtins.attrNames containers);
          # Makes a check from a shell script.
          mkCheck = { name, shellScript, }:
            pkgs.runCommand name { } (''
              # Ensure there is ouput for the derivation (nix will complain otherwise).
              mkdir -p $out
            '' + shellScript);
        };

      });
}
