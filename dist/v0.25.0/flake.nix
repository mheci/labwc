{
  description = "labwc-rs — A Wayland window-stacking compositor (Rust rewrite of labwc)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
          targets = [ "x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu" ];
        };

        buildInputs = with pkgs; [
          wayland wayland-protocols
          libxkbcommon libinput libglvnd
          mesa vulkan-loader vulkan-headers
          cairo pango pixman libpng
          librsvg libxml2 glib
          xorg.libxcb seatd
          udev libdrm
        ];

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config make cmake
          git
        ];

        labwc-rs = pkgs.stdenv.mkDerivation {
          pname = "labwc-rs";
          version = "0.25.0";
          src = self;

          inherit buildInputs nativeBuildInputs;

          buildPhase = ''
            export HOME=$(mktemp -d)
            cargo build --release --locked -p labwc-rs
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/labwc-rs $out/bin/
            mkdir -p $out/share/wayland-sessions
            cp data/labwc.desktop $out/share/wayland-sessions/
            mkdir -p $out/share/labwc
            cp data/rc.xml $out/share/labwc/
          '';

          meta = with pkgs.lib; {
            description = "A Wayland window-stacking compositor (Rust rewrite)";
            license = licenses.gpl2Only;
            platforms = platforms.linux;
            maintainers = [ maintainers.mheci ];
            mainProgram = "labwc-rs";
          };
        };

      in {
        packages = {
          default = labwc-rs;
          labwc-rs = labwc-rs;
        };

        apps.default = flake-utils.lib.mkApp { drv = labwc-rs; };

        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
          shellHook = ''
            echo "labwc-rs development shell"
            echo "  cargo build --release"
            echo "  cargo test"
          '';
        };
      }
    );
}
