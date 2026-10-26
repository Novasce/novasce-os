import type { Recomendado } from "../tipos";

/** A lista vem com a imagem. Para mudar, mude aqui (é a receita do sistema). */
export const RECOMENDADOS: Recomendado[] = [
  { id: "zoom", nome: "Zoom", descricao: "Para reunião com cliente que usa Zoom.", flatpak: "us.zoom.Zoom", ficha: "zoom" },
  { id: "obs", nome: "OBS", descricao: "Gravar a tela para explicar algo ao cliente.", flatpak: "com.obsproject.Studio", ficha: "obs" },
  { id: "gimp", nome: "GIMP", descricao: "Ajustar uma imagem rápida.", flatpak: "org.gimp.GIMP", ficha: "gimp" },
  { id: "libreoffice", nome: "LibreOffice", descricao: "Documentos, planilhas e apresentações.", flatpak: "org.libreoffice.LibreOffice", ficha: "libreoffice" },
  { id: "thunderbird", nome: "Thunderbird", descricao: "O e-mail da Novasce no computador.", flatpak: "org.mozilla.Thunderbird", ficha: "thunderbird" },
  { id: "bitwarden", nome: "Bitwarden", descricao: "As suas senhas.", flatpak: "com.bitwarden.desktop", ficha: "bitwarden" },
];
