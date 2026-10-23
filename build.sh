#!/usr/bin/bash
# Roda dentro da montagem da imagem (Containerfile). Tudo que muda o Aurora
# para virar o Novasce OS passa por aqui. Se um caminho do Aurora mudar, a
# montagem falha de propósito: a máquina segue na versão atual e o erro aparece.
set -euxo pipefail

MARCA=/ctx/marca
TMP=$(mktemp -d)

# --- Pacotes -----------------------------------------------------------------
# Node para o executor dos agentes (precisa de 22 ou mais) e o conversor de SVG
# que gera as imagens da marca a partir do vetor oficial.
dnf5 install -y nodejs npm git librsvg2-tools
node -e 'if (+process.versions.node.split(".")[0] < 22) { console.error("Node < 22"); process.exit(1) }'

# --- Marca: vetores ----------------------------------------------------------
# O cata-vento é o arquivo oficial (design/novasce/marca-vetor), sem redesenho.
# Só a cor muda: branco #F5F5F7 sobre fundo escuro, teal #00666C sobre claro.
sed 's/fill="currentColor"/fill="#F5F5F7"/' "$MARCA/catavento.svg" >"$TMP/catavento-branco.svg"
sed 's/fill="currentColor"/fill="#00666C"/' "$MARCA/catavento.svg" >"$TMP/catavento-teal.svg"

# Papel de parede: área de trabalho, bloqueio e tela de login.
PAPEL=/usr/share/wallpapers/Novasce/contents
mkdir -p "$PAPEL/images"
rsvg-convert -w 3840 -h 2160 "$MARCA/fundo.svg" -o "$PAPEL/images/3840x2160.png"
rsvg-convert -w 1920 -h 1080 "$MARCA/fundo.svg" -o "$PAPEL/images/1920x1080.png"
rsvg-convert -w 400 -h 225 "$MARCA/fundo.svg" -o "$PAPEL/screenshot.png"
# Tudo que apontava para o papel do Aurora passa a mostrar o da Novasce.
test -e /usr/share/wallpapers/Aurora
rm -rf /usr/share/wallpapers/Aurora
ln -s Novasce /usr/share/wallpapers/Aurora

# Logo do sistema ("Sobre este sistema").
install -Dm0644 "$TMP/catavento-teal.svg" /usr/share/icons/hicolor/scalable/apps/novasce-logo.svg
for pasta in /usr/share/icons /usr/share/pixmaps; do
  if [ -d "$pasta" ]; then
    find "$pasta" -name 'distributor-logo.svg' -exec cp "$TMP/catavento-teal.svg" {} \;
  fi
done

# Tela de abertura do KDE (KSplash): cata-vento branco sobre o escuro da casa.
SPLASH=/usr/share/plasma/look-and-feel/dev.getaurora.aurora.desktop/contents/splash
test -f "$SPLASH/images/aurora_logo.svgz"
gzip -9 -n <"$TMP/catavento-branco.svg" >"$SPLASH/images/aurora_logo.svgz"
sed -i 's/color: "black"/color: "#0A2528"/' "$SPLASH/Splash.qml"

# Tela de boot (Plymouth): marca d'água e fundo.
test -d /usr/share/plymouth/themes/spinner
rsvg-convert -h 96 "$TMP/catavento-branco.svg" -o /usr/share/plymouth/themes/spinner/watermark.png
for tema in /usr/share/plymouth/themes/spinner/spinner.plymouth /usr/share/plymouth/themes/bgrt/bgrt.plymouth; do
  if [ -f "$tema" ]; then
    sed -i -E 's/^(BackgroundStartColor|BackgroundEndColor)=.*/\1=0x0a2528/' "$tema"
  fi
done

# --- Marca: tema ---------------------------------------------------------------
# Manual da casa: claro é o padrão, escuro é escolha da pessoa. Destaque teal.
PERFIL=/usr/share/kde-settings/kde-profile/default/xdg/kdeglobals
test -f "$PERFIL"
kwriteconfig6 --file "$PERFIL" --group KDE --key LookAndFeelPackage dev.getaurora.auroralight.desktop
kwriteconfig6 --file "$PERFIL" --group KDE --key ColorScheme BreezeLight
kwriteconfig6 --file "$PERFIL" --group General --key ColorScheme BreezeLight
kwriteconfig6 --file "$PERFIL" --group General --key AccentColor 0,102,108
kwriteconfig6 --file "$PERFIL" --group General --key accentColorFromWallpaper false
kwriteconfig6 --file "$PERFIL" --group Icons --key Theme breeze

# --- Nome do sistema -----------------------------------------------------------
# ID continua "aurora" para o ujust e as ferramentas da base seguirem funcionando.
sed -i \
  -e 's|^NAME=.*|NAME="Novasce OS"|' \
  -e 's|^PRETTY_NAME=.*|PRETTY_NAME="Novasce OS"|' \
  -e 's|^HOME_URL=.*|HOME_URL="https://novasce.com"|' \
  -e 's|^DEFAULT_HOSTNAME=.*|DEFAULT_HOSTNAME="novasce-sede"|' \
  -e '/^LOGO=/d' \
  /usr/lib/os-release
echo 'LOGO=novasce-logo' >>/usr/lib/os-release

# --- Serviços ------------------------------------------------------------------
# A atualização semanal nasce ligada e fica inerte até existir o repositório na
# máquina. O assistente nasce desligado: liga depois do setup e do doctor.
systemctl enable novasce-atualizar.timer

# --- Initramfs -----------------------------------------------------------------
# A tela de boot (inclusive o pedido da senha do disco) mora no initramfs, então
# ele é refeito com a marca nova. Mesmos módulos do Aurora: as configurações do
# dracut dele continuam em /usr/lib/dracut/dracut.conf.d.
KERNEL_VERSION=$(rpm -q --queryformat="%{evr}.%{arch}" kernel-core)
DRACUT_NO_XATTR=1 dracut --kver "$KERNEL_VERSION" --reproducible --force \
  "/usr/lib/modules/$KERNEL_VERSION/initramfs.img"
chmod 0600 "/usr/lib/modules/$KERNEL_VERSION/initramfs.img"

rm -rf "$TMP"
dnf5 clean all
