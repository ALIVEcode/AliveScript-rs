{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    cmake
  ];

  buildInputs = with pkgs; [
    libGL
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXinerama
    xorg.libXrandr
    alsa-lib
    glfw3
    glibc
    clang
    libclang
    # Optional: include these if you are on Wayland
    wayland
    libxkbcommon
  ];

  shellHook = ''
    export LIBCLANG_PATH="${pkgs.lib.makeLibraryPath [
      pkgs.libclang
    ]}"
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
      pkgs.libGL
      pkgs.xorg.libX11
      pkgs.xorg.libXcursor
      pkgs.xorg.libXi
      pkgs.xorg.libXinerama
      pkgs.xorg.libXrandr
      pkgs.alsa-lib
    ]}"
  '';
}
