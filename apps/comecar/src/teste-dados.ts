import type { EstadoTela } from "./tipos";

export const estadoDeTeste: EstadoTela = {
  tela: "passo1", passosFeitos: 0, pessoa: null, boasVindas: null, antesDoPasso1: true, espacos: [], kitDisponivel: false,
  extras: { nuvem: false, senhas: false, voz: false }, manha: true,
  descobertas: { falou: false, pediuAjuda: false, achouNota: false, trocouAtalho: false },
  ias: [], githubConfigurado: true, githubLogin: null, brainRepo: null, movimento: 1, sincronizar: null, googleConectado: false, vozQualquerCampo: true,
};
