{
  description = "Optional development environment for SQLPage";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # Ships the official rustup binaries, so the toolchain in this shell is
    # the same build CI and the release use, down to the bundled LLVM.
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      inherit (nixpkgs) lib;

      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];

      # Oracle governs its own client library by licence, so nixpkgs marks it
      # unfree. Naming it here keeps every other package in this flake free.
      unfreePackages = [ "oracle-instantclient" ];

      forEachSystem =
        build:
        lib.genAttrs systems (
          system:
          build (
            import nixpkgs {
              inherit system;
              config.allowUnfreePredicate = package: builtins.elem (lib.getName package) unfreePackages;
              overlays = [ rust-overlay.overlays.default ];
            }
          )
        );

      cargoEdition = (lib.importTOML ./Cargo.toml).package.edition;

      lockedPlaywrightVersion =
        (lib.importJSON ./package-lock.json).packages."node_modules/playwright-core".version;

      formatterFor =
        pkgs:
        pkgs.treefmt.withConfig {
          runtimeInputs = [
            pkgs.biome
            pkgs.nixfmt
            pkgs.rustfmt
          ];
          settings = {
            on-unmatched = "info";
            formatter = {
              nixfmt = {
                command = "nixfmt";
                includes = [ "*.nix" ];
              };
              rustfmt = {
                command = "rustfmt";
                options = [
                  "--edition"
                  cargoEdition
                ];
                includes = [ "*.rs" ];
              };
              biome = {
                command = "biome";
                options = [
                  "format"
                  "--write"
                  "--no-errors-on-unmatched"
                ];
                includes = [
                  "*.css"
                  "*.js"
                  "*.json"
                  "*.jsonc"
                  "*.mjs"
                  "*.ts"
                ];
              };
            };
          };
        };

      rustToolchainFor = pkgs: pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      # odbc-sys links against a driver manager, and the C sources vendored by
      # aws-lc-sys, libsqlite3-sys and zstd-sys need a compiler. Nothing here
      # reaches for OpenSSL: SQLPage speaks TLS through rustls.
      cargoRuntime = pkgs: [
        (rustToolchainFor pkgs)
        pkgs.stdenv.cc
        pkgs.unixodbc
      ];
      # The database matrix runs prebuilt test binaries that link the unixODBC
      # driver manager statically. It still reads ODBCSYSINI at run time and
      # dlopens whatever absolute path this file names, so pointing that one
      # variable at the store replaces every driver install step.
      odbcDriversFor =
        pkgs:
        pkgs.runCommand "sqlpage-odbc-drivers" { } ''
          mkdir -p "$out"
          oracleDriver=(${lib.getLib pkgs.oracle-instantclient}/lib/libsqora.so.*)
          cat > "$out/odbcinst.ini" <<INI
          [PostgreSQL Unicode]
          Driver = ${pkgs.unixodbcDrivers.psql}/${pkgs.unixodbcDrivers.psql.driver}

          [Oracle 21 ODBC driver]
          Driver = ''${oracleDriver[0]}
          INI
        '';

      cargoEnv = pkgs: {
        LIBRARY_PATH = "${pkgs.unixodbc}/lib";
      };

      # Playwright is installed via npm, we we need to set the browser paths.
      playwrightEnv = pkgs: {
        PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright-driver.browsers}";
        PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = "1";
        PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = "true";
      };

      gatesFor =
        pkgs:
        lib.mapAttrs
          (
            name:
            {
              description,
              packages ? [ ],
              env ? { },
              script,
            }:
            pkgs.writeShellApplication {
              inherit name;
              runtimeInputs = [ pkgs.git ] ++ packages;
              runtimeEnv = env;
              meta = { inherit description; };
              text = ''
                cd "$(git rev-parse --show-toplevel)"
              ''
              + script;
            }
          )
          {
            lint-rust = {
              description = "Check Rust formatting and deny every clippy warning";
              packages = cargoRuntime pkgs;
              env = cargoEnv pkgs;
              script = ''
                cargo fmt --all -- --check
                cargo clippy --all-targets --all-features -- -D warnings
              '';
            };

            test-rust = {
              description = "Run the Rust test suites against DATABASE_URL, or SQLite when it is unset";
              packages = cargoRuntime pkgs;
              env = cargoEnv pkgs;
              script = ''
                cargo test "$@"
              '';
            };

            test-browser = {
              description = "Drive the components in a browser with Playwright";
              packages = [ pkgs.nodejs_26 ] ++ cargoRuntime pkgs;
              env = cargoEnv pkgs // playwrightEnv pkgs;
              script = ''
                npm ci --ignore-scripts
                npm run build
                cd tests/end-to-end
                npx playwright test "$@"
              '';
            };

            test-examples = {
              description = "Run the Hurl suite of an example against its own containers";
              packages = [ pkgs.hurl ];
              script = ''
                scripts/test-examples-hurl.sh "$@"
              '';
            };
          };
    in
    {
      packages = forEachSystem (pkgs: {
        odbc-drivers = odbcDriversFor pkgs;
      });

      formatter = forEachSystem formatterFor;

      checks = forEachSystem (pkgs: {
        formatting = (formatterFor pkgs).check self;
      });

      apps = forEachSystem (
        pkgs:
        lib.mapAttrs (_: gate: {
          type = "app";
          program = lib.getExe gate;
          inherit (gate) meta;
        }) (gatesFor pkgs)
      );

      devShells = forEachSystem (
        pkgs:
        assert lib.assertMsg (lockedPlaywrightVersion == pkgs.playwright-driver.version)
          "package-lock.json pins Playwright ${lockedPlaywrightVersion}, but nixpkgs ships ${pkgs.playwright-driver.version}. Playwright only drives the browsers its own release shipped with, so run `nix flake update` or realign package-lock.json.";
        {
          default = pkgs.mkShell {
            name = "sqlpage";

            packages =
              (with pkgs; [
                biome
                hurl
                nodejs_26
                pkg-config
                sqlite
              ])
              ++ cargoRuntime pkgs
              ++ [ (formatterFor pkgs) ]
              ++ lib.attrValues (gatesFor pkgs);

            buildInputs = with pkgs; [
              unixodbc
              zstd
            ];

            # DATABASE_URL and LISTEN_ON stay unset on purpose. SQLPage reads
            # both from the environment, so exporting them here would override
            # every example and test fixture that ships its own configuration.
            env = {
              RUST_SRC_PATH = "${rustToolchainFor pkgs}/lib/rustlib/src/rust/library";
              ZSTD_SYS_USE_PKG_CONFIG = "1";
            }
            // playwrightEnv pkgs;
          };
        }
      );
    };
}
