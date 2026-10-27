#!/usr/bin/bash
# Confere a imagem montada. Roda DENTRO da imagem, no CI (workflow build):
#   podman run --rm --entrypoint /usr/bin/bash -v "$PWD:/repo:ro" novasce-os:nvidia /repo/tests/imagem.sh nvidia
# Cada linha diz ok ou FALHA. Sai com erro se alguma falhar.
set -uo pipefail

VARIANTE=${1:?passe a variante: nvidia ou padrao}
REPO=${REPO:-/repo}
falhas=0

conferir() {
  local nome=$1
  shift
  if "$@" >/dev/null 2>&1; then
    printf 'ok    %s\n' "$nome"
  else
    printf 'FALHA %s\n' "$nome"
    falhas=$((falhas + 1))
  fi
}

PERFIL=/usr/share/kde-settings/kde-profile/default/xdg
LNF=/usr/share/plasma/look-and-feel/com.novasce.desktop
TEMA=/usr/share/plymouth/themes/novasce
KVER=$(rpm -q --queryformat '%{evr}.%{arch}' kernel-core)

echo "== Nome e versão"
conferir "os-release diz Novasce OS" grep -qx 'NAME="Novasce OS"' /usr/lib/os-release
conferir "os-release mantém ID=aurora" grep -qx 'ID=aurora' /usr/lib/os-release
conferir "IMAGE_VERSION no formato AAAA.MM.DD.N" grep -qE '^IMAGE_VERSION="[0-9]{4}\.[0-9]{2}\.[0-9]{2}\.[0-9]+"$' /usr/lib/os-release
conferir "variante gravada" grep -qx "$VARIANTE" /usr/share/novasce/variante
conferir "image-info aponta para a Novasce" jq -e '."image-ref" == "ostree-image-signed:docker://ghcr.io/novasce/novasce-os"' /usr/share/ublue-os/image-info.json
conferir "Sobre este sistema" grep -qx 'Name=Novasce OS' "$PERFIL/kcm-about-distrorc"

echo "== Marca"
conferir "papel de parede 16:9" test -s /usr/share/wallpapers/Novasce/contents/images/3840x2160.png
conferir "papel de parede 16:10" test -s /usr/share/wallpapers/Novasce/contents/images/3840x2400.png
conferir "o papel do Aurora aponta para o da Novasce" test "$(readlink /usr/share/wallpapers/Aurora)" = Novasce
conferir "conta nova nasce com o papel da Novasce" grep -qx 'Image=Novasce' "$LNF/contents/defaults"
conferir "conta que já existia ganha o papel da Novasce" grep -q 'wallpapers/Novasce' /usr/share/plasma/shells/org.kde.plasma.desktop/contents/updates/91-novasce-papel-de-parede.js
conferir "logo teal do menu" grep -q 'fill="#00666C"' /usr/share/icons/hicolor/scalable/apps/novasce-logo.svg
conferir "ícone start-here é o cata-vento" grep -q 'fill="#00666C"' /usr/share/icons/hicolor/scalable/places/distributor-logo-symbolic.svg
conferir "fonte Inter instalada" sh -c 'fc-list : family | grep -q "^Inter"'

echo "== Boot"
conferir "tema de boot é o novasce" grep -qx 'Theme=novasce' /etc/plymouth/plymouthd.conf
conferir "plugin script instalado" test -f /usr/lib64/plymouth/script.so
for peca in catavento trilho linha ponto nome pedido; do
  conferir "boot: $peca.png" test -s "$TEMA/$peca.png"
done
conferir "initramfs leva o tema" sh -c "lsinitrd /usr/lib/modules/$KVER/initramfs.img | grep -q 'usr/share/plymouth/themes/novasce/novasce.script'"

echo "== Área de trabalho"
conferir "look-and-feel Novasce" test -f "$LNF/metadata.json"
conferir "abertura com a linha pelo stage" grep -q 'stage / 6' "$LNF/contents/splash/Splash.qml"
conferir "abertura não gira" sh -c "! grep -q RotationAnimator $LNF/contents/splash/Splash.qml"
conferir "abertura com o cata-vento" test -s "$LNF/contents/splash/images/catavento.svgz"
conferir "layout B com menu global" grep -q 'org.kde.plasma.appmenu' "$LNF/contents/layouts/org.kde.plasma.desktop-layout.js"
conferir "perfil usa o look-and-feel Novasce" grep -qx 'LookAndFeelPackage=com.novasce.desktop' "$PERFIL/kdeglobals"
conferir "perfil claro com destaque teal" grep -qx 'AccentColor=0,102,108' "$PERFIL/kdeglobals"
conferir "perfil traz as cores do esquema" grep -q '^\[Colors:Selection\]' "$PERFIL/kdeglobals"
conferir "abertura Novasce no perfil" grep -qx 'Theme=com.novasce.desktop' "$PERFIL/ksplashrc"
conferir "bloqueio com o papel teal" grep -q 'wallpapers/Novasce' "$PERFIL/kscreenlockerrc"
conferir "login com o papel teal" grep -q 'wallpapers/Novasce' /usr/lib/plasmalogin/defaults.conf
conferir "esquema Novasce Claro" grep -qx 'Name=Novasce Claro' /usr/share/color-schemes/NovasceClaro.colors
conferir "esquema Novasce Grafite" grep -qx 'BackgroundNormal=30,30,32' /usr/share/color-schemes/NovasceGrafite.colors

echo "== Apps"
conferir "novasce-sinal roda" /usr/bin/novasce-sinal --version
conferir "URL da Sala vem da config" test "$(/usr/bin/novasce-sinal url sala)" = "https://tenancy.novasce.com/sala"
conferir "novasce-helper roda" /usr/libexec/novasce/novasce-helper --help
conferir "Preparar instalado" test -x /usr/bin/novasce-preparar
conferir "Preparar acha todas as bibliotecas" sh -c '! ldd /usr/bin/novasce-preparar | grep -q "not found"'
for f in /usr/share/applications/novasce-*.desktop /etc/xdg/autostart/novasce-*.desktop; do
  conferir "atalho válido: ${f##*/}" desktop-file-validate "$f"
done
# PNG de 256 px: a largura mora nos bytes 16 a 19 do arquivo (00 00 01 00).
for app in tenancy claude chatgpt gemini whatsapp; do
  conferir "ícone de verdade: $app" sh -c "test \"\$(od -An -tx1 -j16 -N4 /usr/share/icons/hicolor/256x256/apps/novasce-$app.png | tr -d ' \n')\" = 00000100"
done
conferir "nenhum ícone com a letra no quadrado" sh -c "! grep -lqs '<text' /usr/share/icons/hicolor/scalable/apps/novasce-*.svg"
for app in tenancy claude chatgpt gemini whatsapp material; do
  icone=$(sed -n 's/^Icon=//p' "/usr/share/applications/novasce-$app.desktop")
  conferir "o ícone do dock acha arquivo: $app ($icone)" sh -c "ls /usr/share/icons/*/*/*/$icone.* 2>/dev/null | grep -q ."
done
for app in com.google.Chrome md.obsidian.Obsidian com.bitwarden.desktop; do
  conferir "pré-instala $app" grep -q "\[Flatpak Preinstall $app\]" /usr/share/flatpak/preinstall.d/novasce-*.preinstall
done
conferir "Chrome é o navegador do KDE" grep -qx 'BrowserApplication=com.google.Chrome.desktop' "$PERFIL/kdeglobals"
for lista in /etc/xdg/mimeapps.list "$PERFIL/mimeapps.list" "$PERFIL/kde-mimeapps.list"; do
  conferir "Chrome abre https em ${lista}" grep -qx 'x-scheme-handler/https=com.google.Chrome.desktop;' "$lista"
done

echo "== Papéis, contas e agentes"
conferir "política do ajudante" xmllint --noout /usr/share/polkit-1/actions/com.novasce.helper.policy
conferir "ajudante só para wheel" grep -q 'isInGroup("wheel")' /usr/share/polkit-1/rules.d/50-novasce-helper.rules
for f in novasce-agentes.sysusers novasce-agentes.tmpfiles 50-novasce-sede-sem-suspender.conf 50-novasce-sede-autologin.conf kscreenlockerrc powerdevilrc kwalletrc ksmserverrc chrome-flags.conf org.kde.konsole.desktop; do
  conferir "arquivo da sede: $f" test -s "/usr/share/novasce/sede/$f"
done
conferir "agentes não nascem em todo notebook" test ! -e /usr/lib/sysusers.d/novasce-agentes.conf
conferir "executor só na sede" grep -q 'ExecCondition=/usr/bin/grep -qx sede /etc/novasce/papel' /usr/lib/systemd/system/novasce-assistente.service
conferir "executor marca a rodada" grep -q 'rodada.lock' /usr/lib/systemd/system/novasce-assistente.service
conferir "sinal ligado" test "$(systemctl --root=/ is-enabled novasce-sinal.timer 2>/dev/null)" = enabled
conferir "executor desligado até o doctor" test "$(systemctl --root=/ is-enabled novasce-assistente.timer 2>/dev/null)" != enabled
conferir "Sala com página de reserva" grep -q 'A Sala chega com o Tenancy' /usr/share/novasce/sala-reserva/index.html
conferir "reserva da Sala numa pasta que o Chrome do Flatpak enxerga" sh -c "grep -q 'RESERVA_PASTA=\"\$HOME/.var/app/' /usr/libexec/novasce/sala && ! grep -q 'file:///usr/share' /usr/libexec/novasce/sala"
conferir "Chrome da Sala sem a carteira do KDE" grep -q -- '--password-store=basic' /usr/libexec/novasce/sala
conferir "Chrome da conta Sede sem a carteira do KDE" grep -qx -- '--password-store=basic' /usr/share/novasce/sede/chrome-flags.conf
conferir "carteira do KDE desligada na Sede" grep -qx 'Enabled=false' /usr/share/novasce/sede/kwalletrc
conferir "Sede abre sempre vazia" grep -qx 'loginMode=emptySession' /usr/share/novasce/sede/ksmserverrc
conferir "Konsole escondido na Sede" grep -qx 'Hidden=true' /usr/share/novasce/sede/org.kde.konsole.desktop
conferir "Sala em perfil que persiste, sem modo anônimo" sh -c "grep -q 'user-data-dir' /usr/libexec/novasce/sala && ! grep -q incognito /usr/libexec/novasce/sala"
conferir "o sistema não guarda token de tela" sh -c "! grep -rqs 'tela.json' /usr/libexec/novasce /usr/share/novasce /etc/xdg/autostart"
conferir "nenhuma máquina registrada na imagem" test ! -e /var/lib/novasce/maquina.json
conferir "nenhum papel gravado na imagem" test ! -e /etc/novasce/papel

echo "== Assinatura e atualização"
conferir "chave pública da casa" cmp "$REPO/cosign.pub" /usr/lib/pki/containers/novasce-os.pub
conferir "política exige a assinatura da Novasce" jq -e '.transports.docker["ghcr.io/novasce/novasce-os"][0] | .type == "sigstoreSigned" and .keyPath == "/usr/lib/pki/containers/novasce-os.pub"' /etc/containers/policy.json
conferir "a base do Aurora continua confiável" jq -e '.transports.docker["ghcr.io/ublue-os"][0].type == "sigstoreSigned"' /etc/containers/policy.json
conferir "padrão recusa o que não tem regra" jq -e '.default[0].type == "reject"' /etc/containers/policy.json
conferir "anexos sigstore ligados" grep -q 'use-sigstore-attachments: true' /etc/containers/registries.d/novasce-os.yaml
conferir "atualização semanal" grep -qx 'OnCalendar=Sun \*-\*-\* 04:00:00' /usr/lib/systemd/system/uupd.timer.d/50-novasce-semanal.conf

echo
if [ "$falhas" -eq 0 ]; then
  echo "Tudo certo."
else
  echo "$falhas conferência(s) falharam."
  exit 1
fi
