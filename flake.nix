{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      naersk,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        lib = pkgs.lib;
        rust = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "rust-src"
            "rustfmt"
            "clippy"
            "rust-analyzer"
            "llvm-tools"
          ];
          targets = [
            "aarch64-unknown-none-softfloat"
          ];
        };
        naersk' = pkgs.callPackage naersk {
          cargo = rust;
          rustc = rust;
        };
        buildKernel =
          {
            bsp ? "rpi4",
            mode ? "build",
          }:
          naersk'.buildPackage {
            src = ./.;
            release = false;
            cargoBuildOptions =
              opts:
              opts
              ++ [
                "--no-default-features"
                "--features"
                "bsp_${bsp}"
              ];
            inherit mode;
          };
      in
      {
        packages = rec {
          default = kernel;
          # build the kernel
          kernel = buildKernel { };
          test = buildKernel { mode = "test"; };
          fmt = buildKernel { mode = "fmt"; };
          check = buildKernel { mode = "check"; };
          clippy = buildKernel { mode = "clippy"; };
          # run the kernel with qemu
          qemu = pkgs.writeShellScriptBin "qemu" ''
            ${pkgs.qemu}/bin/qemu-system-aarch64 \
            --nographic \
            -M raspi4b \
            -kernel ${kernel}/bin/kernel \
            -m 2G \
            -smp 4 \
            -s \
            -S \
          '';
          gdb = pkgs.writeShellScriptBin "gdb" ''
            ${lib.getExe pkgs.gdb} \
            --iex="file ${kernel}/bin/kernel" \
            --iex="target remote localhost:1234" \
            --iex="layout src" \
            --iex="b _start_rust" \
          '';
          objdump = pkgs.writeShellScriptBin "objdump" ''
            ${pkgs.cargo-binutils}/bin/rust-objdump \
            --disassemble  \
            --demangle \
            --section .text \
            --section .rodata \
            ${kernel}/bin/kernel
          '';
        };
        devShell = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.cargo-binutils
            pkgs.gdb
            pkgs.qemu
          ];
        };
      }
    );
}
