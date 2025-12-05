# To learn more about how to use Nix to configure your environment
# see: https://firebase.google.com/docs/studio/customize-workspace
{ pkgs, ... }: {
  # Which nixpkgs channel to use.
  channel = "stable-24.05"; # or "unstable"

  # Use https://search.nixos.org/packages to find packages
  packages = [
    pkgs.gcc
    pkgs.rustc
    pkgs.cargo
    pkgs.openssl
    pkgs.pkg-config
    pkgs.zlib
    pkgs.openssl.dev
    pkgs.go
    pkgs.python311
    pkgs.python311Packages.pip
    pkgs.docker-compose
    pkgs.sudo

    # pkgs.nodejs_20 
    # pkgs.nodePackages.nodemon
    pkgs.docker
    pkgs.gnumake
    (pkgs.google-cloud-sdk.withExtraComponents [
      pkgs.google-cloud-sdk.components.gcloud
      pkgs.google-cloud-sdk.components.cloud-datastore-emulator
    ])
  ];

  services.docker.enable = true;
}
