#!/usr/bin/bash
# Ferramentas que os apps da casa chamam. Falha de propósito se faltar alguma.
set -euxo pipefail
dnf5 install -y gitleaks rclone ydotool wl-clipboard pulseaudio-utils gtk-layer-shell webkit2gtk4.1 \
  rsms-inter-fonts jetbrains-mono-fonts-all spectacle librsvg2-tools
gitleaks version
# a flag é do comando bisync: "rclone help flags" só lista as globais
rclone bisync --help | grep -- "--conflict-loser" >/dev/null
for c in ydotool ydotoold wl-copy wl-paste parecord spectacle rsvg-convert busctl; do command -v "$c"; done
