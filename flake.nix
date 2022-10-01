{
  description = "Build a cargo project without extra checks";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        toolchain = pkgs.rust-bin.nightly.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
          extensions = [ "rust-analyzer" "rust-src" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        cosmwebwasm =
          craneLib.buildPackage { src = craneLib.cleanCargoSource ./.; };
      in {
        packages.default = cosmwebwasm;
        apps.default = flake-utils.lib.mkApp { drv = cosmwebwasm; };
        devShells.default = pkgs.mkShell { buildInputs = [ pkgs.curl toolchain pkgs.darwin.apple_sdk.frameworks.Security]; };
      });
}
