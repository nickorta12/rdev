{
  description = "rdev remote-development helper";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachSystem [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ] (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rdev";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [
            pkgs.installShellFiles
          ];

          postInstall = ''
            installShellCompletion --cmd rdev \
              --bash <($out/bin/rdev completions bash) \
              --fish <($out/bin/rdev completions fish) \
              --zsh <($out/bin/rdev completions zsh)
          '';
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            clippy
            mutagen
            openssh
            pkg-config
            rsync
            rustc
            rustfmt
          ];
        };
      });
}
