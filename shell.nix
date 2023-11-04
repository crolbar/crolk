{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
    nativeBuildInputs = with pkgs; [
        glib
        gtk3
        pkg-config
        libappindicator-gtk3
    ];
}
