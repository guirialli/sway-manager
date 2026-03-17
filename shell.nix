{ pkgs ? import <nixpkgs> {} }:

let
  libs = with pkgs; [
    brotli krb5 zstd zlib xz

    freetype fontconfig glib libGL libglvnd libxkbcommon dbus

    wayland xorg.libX11 xorg.libxcb xorg.xcbutil xorg.xcbutilcursor
    xorg.xcbutilkeysyms xorg.xcbutilimage xorg.xcbutilrenderutil xorg.xcbutilwm

    stdenv.cc.cc.lib
  ];
in
pkgs.mkShell {
  buildInputs = [ 
    pkgs.python3
    pkgs.gcc  
    pkgs.scons 
    pkgs.binutils
  ] ++ libs;

  shellHook = ''
    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libs}:$LD_LIBRARY_PATH
    
    export C_INCLUDE_PATH="${pkgs.python3}/include/python${pkgs.python3.pythonVersion}:$C_INCLUDE_PATH"
    export CPLUS_INCLUDE_PATH="${pkgs.python3}/include/python${pkgs.python3.pythonVersion}:$CPLUS_INCLUDE_PATH"
    
  '';
}
