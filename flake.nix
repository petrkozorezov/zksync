rec {
  description = "zksync dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-21.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      system = "x86_64-linux";
      pkgs   = nixpkgs.legacyPackages.${system};
    in {
        devShell.${system} = pkgs.mkShell {
          shellHook =
            ''
              export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
              rustup toolchain install stable
              export ZKSYNC_HOME=`pwd`
              export PATH=$ZKSYNC_HOME/bin:$PATH
              echo "Hello zk shell"
              echo "Type 'zk && zk init' to prepare"
              echo "Type 'zk server' to start all server components"
            '';

          buildInputs = with pkgs; [
              gnumake
              git
              cacert
              docker-compose
              nodejs
              yarn
              axel
              rustup
              lld
              postgresql
              diesel-cli
              sqlx-cli
              solc
              cmake
              zlib
              binutils-unwrapped
              wasm-pack
              binaryen
              pkg-config
              openssl.dev
            ];
        };
    };
}
