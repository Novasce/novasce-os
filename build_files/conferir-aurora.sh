#!/usr/bin/bash
# Os caminhos do Aurora que a Novasce substitui (lidos em 23/10/2026).
# Se um deles mudar, a montagem para aqui de propósito: a máquina segue na
# versão anterior e o erro aparece no CI (spec, risco 2).
set -uo pipefail

falhou=0
precisa() {
  if ! "$@"; then
    echo "O Aurora mudou: falhou '$*'" >&2
    falhou=1
  fi
}

precisa test -L /usr/share/wallpapers/Aurora
precisa test -f /usr/lib/plasmalogin/defaults.conf
precisa test -f /usr/share/kde-settings/kde-profile/default/xdg/kdeglobals
precisa test -f /usr/share/kde-settings/kde-profile/default/xdg/ksplashrc
precisa test -f /usr/share/kde-settings/kde-profile/default/xdg/kscreenlockerrc
precisa test -f /usr/share/kde-settings/kde-profile/default/xdg/kcm-about-distrorc
precisa test -d /usr/share/plasma/look-and-feel/dev.getaurora.aurora.desktop
precisa test -f /usr/share/icons/hicolor/scalable/places/distributor-logo-symbolic.svg
precisa test -f /usr/share/color-schemes/BreezeLight.colors
precisa test -f /usr/share/color-schemes/BreezeDark.colors
precisa test -f /etc/containers/policy.json
precisa test -f /usr/share/ublue-os/image-info.json
precisa test -f /usr/lib/systemd/system/uupd.timer
precisa grep -qx 'ID=aurora' /usr/lib/os-release
precisa command -v kwriteconfig6
precisa command -v jq

exit "$falhou"
