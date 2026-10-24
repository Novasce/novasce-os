#!/usr/bin/bash
# KDE da Novasce: cata-vento da abertura, esquemas de cor e as cores no perfil.
set -euo pipefail

LNF=/usr/share/plasma/look-and-feel/com.novasce.desktop
PERFIL=/usr/share/kde-settings/kde-profile/default/xdg
CORES=/usr/share/color-schemes

test -f "$LNF/metadata.json"
test -f "$LNF/contents/splash/Splash.qml"
test -f "$LNF/contents/layouts/org.kde.plasma.desktop-layout.js"
if grep -q RotationAnimator "$LNF/contents/splash/Splash.qml"; then
  echo "A abertura não pode girar nada." >&2
  exit 1
fi

# Cata-vento branco da abertura, do vetor oficial.
install -d "$LNF/contents/splash/images"
sed 's/fill="currentColor"/fill="#F5F5F7"/' /ctx/marca/catavento.svg | gzip -9 -n >"$LNF/contents/splash/images/catavento.svgz"

cor() { kwriteconfig6 --file "$1" --group "$2" --key "$3" "$4"; }
GRUPOS=(Colors:Button Colors:View Colors:Window Colors:Selection Colors:Tooltip Colors:Complementary Colors:Header)

# Novasce Claro: o Breeze claro com o teal da casa no lugar do azul.
C="$CORES/NovasceClaro.colors"
cp "$CORES/BreezeLight.colors" "$C"
cor "$C" General Name "Novasce Claro"
cor "$C" General ColorScheme NovasceClaro
for g in "${GRUPOS[@]}"; do
  cor "$C" "$g" DecorationFocus "0,102,108"
  cor "$C" "$g" DecorationHover "0,102,108"
done
cor "$C" Colors:Selection BackgroundNormal "0,102,108"
cor "$C" Colors:Selection BackgroundAlternate "0,77,82"
cor "$C" Colors:Selection ForegroundNormal "255,255,255"
cor "$C" Colors:View ForegroundLink "0,102,108"
cor "$C" Colors:Window ForegroundLink "0,102,108"

# Novasce Grafite (D Grafite): fundo #1E1E20 e destaque ciano #00C4CC.
G="$CORES/NovasceGrafite.colors"
cp "$CORES/BreezeDark.colors" "$G"
cor "$G" General Name "Novasce Grafite"
cor "$G" General ColorScheme NovasceGrafite
for g in Colors:Window Colors:Header Colors:Complementary; do
  cor "$G" "$g" BackgroundNormal "30,30,32"
  cor "$G" "$g" BackgroundAlternate "38,38,42"
done
for g in Colors:View Colors:Button Colors:Tooltip; do
  cor "$G" "$g" BackgroundNormal "38,38,42"
  cor "$G" "$g" BackgroundAlternate "44,44,48"
done
for g in "${GRUPOS[@]}"; do
  cor "$G" "$g" DecorationFocus "0,196,204"
  cor "$G" "$g" DecorationHover "0,196,204"
done
cor "$G" Colors:Selection BackgroundNormal "0,196,204"
cor "$G" Colors:Selection BackgroundAlternate "0,150,156"
cor "$G" Colors:Selection ForegroundNormal "10,37,40"
cor "$G" Colors:View ForegroundLink "0,196,204"
cor "$G" Colors:Window ForegroundLink "0,196,204"
cor "$G" WM activeBackground "30,30,32"
cor "$G" WM inactiveBackground "30,30,32"

# Só o nome do esquema não pinta uma conta nova: as cores entram no kdeglobals
# do perfil, que é a base de toda conta criada.
awk '/^\[/{ dentro = ($0 ~ /^\[(Colors:|WM\])/) } dentro' "$C" >>"$PERFIL/kdeglobals"
grep -q '^\[Colors:Selection\]' "$PERFIL/kdeglobals"
grep -qx 'LookAndFeelPackage=com.novasce.desktop' "$PERFIL/kdeglobals"
