// Novasce OS · área de trabalho B (canvas v0.5, prancha 3).
// Barra fina no alto: menu com o cata-vento, menu global, bandeja e relógio.
// Dock flutuante embaixo, com os apps da casa. Os quatro sinais da barra
// (falar, nuvem, senhas, ajuda) e o Começar no dock vêm do M3.

const barra = new Panel();
barra.location = "top";
barra.height = 30;
barra.floating = false;
barra.hiding = "none";

const menu = barra.addWidget("org.kde.plasma.kickoff");
menu.currentConfigGroup = ["General"];
menu.writeConfig("icon", "novasce-logo");

barra.addWidget("org.kde.plasma.appmenu");
barra.addWidget("org.kde.plasma.panelspacer");
const bandeja = barra.addWidget("org.kde.plasma.systemtray");
bandeja.currentConfigGroup = ["General"];
// Os quatro sinais do M3, sempre visíveis e nesta ordem (falar, nuvem, senhas, ajuda),
// mais o "pronta" da atualização quando existir.
bandeja.writeConfig("shownItems", ["novasce-0-atualizacao", "novasce-1-falar", "novasce-2-nuvem", "novasce-3-senhas", "novasce-4-ajuda"]);

const relogio = barra.addWidget("org.kde.plasma.digitalclock");
relogio.currentConfigGroup = ["Appearance"];
relogio.writeConfig("showDate", true);
relogio.writeConfig("dateFormat", "shortDate");

const dock = new Panel();
dock.location = "bottom";
dock.height = 62;
dock.floating = true;
dock.alignment = "center";
dock.lengthMode = "fit";
dock.hiding = "none";

const apps = dock.addWidget("org.kde.plasma.icontasks");
apps.currentConfigGroup = ["General"];
apps.writeConfig("launchers", [
  "applications:novasce-comecar.desktop",
  "applications:novasce-tenancy.desktop",
  "applications:novasce-claude.desktop",
  "applications:novasce-chatgpt.desktop",
  "applications:novasce-gemini.desktop",
  "applications:md.obsidian.Obsidian.desktop",
  "applications:novasce-material.desktop",
  "applications:com.google.Chrome.desktop",
  "applications:novasce-whatsapp.desktop",
  "applications:com.bitwarden.desktop.desktop",
]);
apps.writeConfig("showOnlyCurrentDesktop", false);
apps.reloadConfig();
