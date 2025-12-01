{ pkgs, ... }: {
  # Add the Go runtime environment
  packages = [
    pkgs.go
    pkgs.python311
    pkgs.python311Packages.pip
  ];
}
