{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };
  outputs = { self, flake-utils, nixpkgs, ... } @ inputs:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let pkgs = import nixpkgs
        {
          inherit system;
          config.allowUnfree = true;
          overlays = [
            inputs.rust-overlay.overlay
          ];
        }; in
      {
        devShell = with pkgs; mkShell {
          buildInputs = [
            rnix-lsp

            clang_13
            lld_13
            lldb_13
            rust-analyzer
            rust-bin.stable.latest.default
          ];
        };
      }
    )
  ;
}
