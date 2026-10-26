#!/usr/bin/bash
# Marca da Novasce: papel de parede, logos e o nome do sistema.
set -euo pipefail

MARCA=/ctx/marca
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# Cata-vento oficial (design/novasce/marca-vetor), sem redesenho. Só a cor muda.
sed 's/fill="currentColor"/fill="#00666C"/' "$MARCA/catavento.svg" >"$TMP/catavento-teal.svg"
sed 's/fill="currentColor"/fill="#F5F5F7"/' "$MARCA/catavento.svg" >"$TMP/catavento-branco.svg"
grep -q 'fill="#00666C"' "$TMP/catavento-teal.svg"

# Papel de parede teal aprovado em 07/10 (design/novasce/wallpaper-e-banner/saida).
PAPEL=/usr/share/wallpapers/Novasce/contents/images
install -Dm0644 "$MARCA/wallpaper-teal-16x9.png" "$PAPEL/3840x2160.png"
install -Dm0644 "$MARCA/wallpaper-teal-16x10.png" "$PAPEL/3840x2400.png"
test -f /usr/share/wallpapers/Novasce/metadata.json
# Tudo que apontava para o papel do Aurora (área de trabalho, bloqueio, login)
# passa a mostrar o da Novasce.
rm /usr/share/wallpapers/Aurora
ln -s Novasce /usr/share/wallpapers/Aurora

# Logos: menu (start-here aponta para distributor-logo-symbolic), "Sobre" e ícone do sistema.
install -Dm0644 "$TMP/catavento-teal.svg" /usr/share/icons/hicolor/scalable/apps/novasce-logo.svg
install -m0644 "$TMP/catavento-teal.svg" /usr/share/icons/hicolor/scalable/distributor-logo.svg
install -m0644 "$TMP/catavento-teal.svg" /usr/share/icons/hicolor/scalable/places/distributor-logo-symbolic.svg
rsvg-convert -w 256 -h 256 --keep-aspect-ratio "$TMP/catavento-teal.svg" -o /usr/share/pixmaps/system-logo.png
rsvg-convert -w 256 -h 256 --keep-aspect-ratio "$TMP/catavento-branco.svg" -o /usr/share/pixmaps/system-logo-white.png
gtk-update-icon-cache -f /usr/share/icons/hicolor 2>/dev/null || true

# Nome do sistema. O ID continua "aurora": o ujust e as ferramentas da base dependem dele.
OS=/usr/lib/os-release
sed -i \
  -e 's|^NAME=.*|NAME="Novasce OS"|' \
  -e "s|^PRETTY_NAME=.*|PRETTY_NAME=\"Novasce OS ${IMAGE_VERSION}\"|" \
  -e 's|^HOME_URL=.*|HOME_URL="https://novasce.com"|' \
  -e 's|^DOCUMENTATION_URL=.*|DOCUMENTATION_URL="https://github.com/Novasce/novasce-os"|' \
  -e 's|^SUPPORT_URL=.*|SUPPORT_URL="https://tenancy.novasce.com"|' \
  -e 's|^BUG_REPORT_URL=.*|BUG_REPORT_URL="https://github.com/Novasce/novasce-os/issues"|' \
  -e 's|^DEFAULT_HOSTNAME=.*|DEFAULT_HOSTNAME="novasce"|' \
  -e '/^IMAGE_VERSION=/d' \
  -e '/^LOGO=/d' \
  "$OS"
{
  echo "IMAGE_VERSION=\"${IMAGE_VERSION}\""
  echo 'LOGO=novasce-logo'
} >>"$OS"
grep -qx 'ID=aurora' "$OS"
grep -qx 'NAME="Novasce OS"' "$OS"

# A imagem diz de onde vem (o ujust e o uupd leem este arquivo).
INFO=/usr/share/ublue-os/image-info.json
jq --arg variante "$VARIANTE" '
  ."image-name" = "novasce-os"
  | ."image-vendor" = "novasce"
  | ."image-ref" = "ostree-image-signed:docker://ghcr.io/novasce/novasce-os"
  | ."image-tag" = ("stable-" + $variante)' "$INFO" >"$TMP/info.json"
install -m0644 "$TMP/info.json" "$INFO"

# A variante desta imagem: o Preparar compara com a placa encontrada.
install -d /usr/share/novasce
printf '%s\n' "$VARIANTE" >/usr/share/novasce/variante
