{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Macroquad / X11 dependencies
    pkg-config
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    xorg.libXinerama
    libxkbcommon
    libGL
  ];

  # This tells the dynamic linker where the .so files are
  shellHook = ''
    export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
      pkgs.xorg.libX11
      pkgs.xorg.libXcursor
      pkgs.xorg.libXrandr
      pkgs.xorg.libXi
      pkgs.xorg.libXinerama
      pkgs.libxkbcommon
      pkgs.libGL
    ]}:$LD_LIBRARY_PATH"
  '';
}

