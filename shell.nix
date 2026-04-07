{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    cargo
    rustc
    rustfmt
    clippy
  ];

  buildInputs = with pkgs; [
    openssl
  ];
}
