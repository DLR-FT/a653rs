{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    devshell.url = "github:numtide/devshell";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, utils, devshell, fenix, ... }@inputs:
    utils.lib.eachSystem [ "aarch64-linux" "i686-linux" "x86_64-linux" ]
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ devshell.overlays.default ];
          };
          rust-toolchain = with fenix.packages.${system};
            combine [
              stable.rustc
              stable.cargo
              stable.clippy
              latest.rustfmt
              # rust-analyzer
              targets.thumbv6m-none-eabi.stable.rust-std
            ];
        in
        rec {
          devShells.default = (pkgs.devshell.mkShell {
            imports = [ "${devshell}/extra/git/hooks.nix" ];
            name = "a653rs-dev-shell";
            packages = with pkgs; [
              clang
              rust-toolchain
              cargo-outdated
              cargo-udeps
              cargo-audit
              cargo-expand
              cargo-all-features
              cargo-watch
              cargo-release
              nixpkgs-fmt
              rust-analyzer
            ];
            git.hooks = {
              enable = true;
              pre-commit.text = ''
                # echo "Build all feature combinations:"
                # RUSTFLAGS=-Awarnings verify-features
                # echo "Build no_std:"
                # RUSTFLAGS=-Awarnings verify-no_std
                # echo "Build documentation:"
                # RUSTFLAGS=-Awarnings verify-doc
                # echo "Run 'nix flake check'"
                nix flake check
              '';
            };
            commands = [
              { package = "git-cliff"; }
              { package = "treefmt"; }
              {
                name = "udeps";
                command = ''
                  PATH=${fenix.packages.${system}.latest.rustc}/bin:$PATH
                  cargo udeps $@
                '';
                help = pkgs.cargo-udeps.meta.description;
              }
              {
                name = "outdated";
                command = "cargo outdated $@";
                help = pkgs.cargo-outdated.meta.description;
              }
              {
                name = "audit";
                command = "cargo audit $@";
                help = pkgs.cargo-audit.meta.description;
              }
              {
                name = "expand";
                command = ''
                  PATH=${fenix.packages.${system}.latest.rustc}/bin:$PATH
                  cargo expand $@
                '';
                help = pkgs.cargo-expand.meta.description;
              }
              {
                name = "verify-no_std";
                command = ''
                  cd $PRJ_ROOT
                  cargo check-all-features --target thumbv6m-none-eabi $@
                '';
                help =
                  "Verify that the library builds for no_std without std-features";
                category = "test";
              }
              {
                name = "verify-doc";
                command = ''
                  cd $PRJ_ROOT
                  cargo doc --all-features --workspace $@
                  cargo doc --no-default-features --workspace $@
                '';
                help =
                  "Verify that the documentation builds without problems";
                category = "test";
              }
              {
                name = "verify-features";
                command = ''
                  cd $PRJ_ROOT
                  cargo check-all-features $@
                '';
                help =
                  "Verify that a653rs builds for all feature combinations";
                category = "test";
              }
              {
                name = "verify-tests";
                command = ''
                  cd $PRJ_ROOT
                  cargo test-all-features $@
                '';
                help =
                  "Verify that a653rs tests run for all feature combinations";
                category = "test";
              }
              {
                name = "verify-examples";
                command = ''
                  cd $PRJ_ROOT
                  cargo check-all-features --examples $@
                '';
                help =
                  "Verify that a653rs examples run for all feature combinations";
                category = "test";
              }
            ];
          });
          checks = {
            nixpkgs-fmt = pkgs.runCommand "nixpkgs-fmt"
              {
                nativeBuildInputs = [ pkgs.nixpkgs-fmt ];
              } "nixpkgs-fmt --check ${./.}; touch $out";
            cargo-fmt = pkgs.runCommand "cargo-fmt"
              {
                nativeBuildInputs = [ rust-toolchain ];
              } "cd ${./.}; cargo fmt --check; touch $out";
          };
        });
}

