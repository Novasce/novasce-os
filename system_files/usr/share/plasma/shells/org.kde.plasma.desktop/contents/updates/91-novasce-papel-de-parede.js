// Novasce OS: o papel de parede da Novasce em toda área de trabalho, uma vez por conta.
// Pega também a conta que nasceu no Aurora antes da troca para a imagem (o "eduardo" da
// sede, 26/10). O Plasma roda cada arquivo desta pasta uma vez e anota no plasmashellrc.
for (const d of desktops()) {
  d.wallpaperPlugin = "org.kde.image";
  d.currentConfigGroup = ["Wallpaper", "org.kde.image", "General"];
  d.writeConfig("Image", "file:///usr/share/wallpapers/Novasce/");
  d.reloadConfig();
}
