import type { Codigo, Espera, EstadoTela, OrigemBrain } from "./tipos";

const base: EstadoTela = {
  tela: "passo1", passosFeitos: 0, pessoa: null, boasVindas: null, antesDoPasso1: true,
  espacos: [
    { slug: "studio", nome: "Studio", descricao: "o institucional da Novasce · todo o time participa", estado: "trazer" },
    { slug: "food", nome: "Food", descricao: "não é do seu setor no Tenancy", estado: "fora" },
  ],
  kitDisponivel: false, extras: { nuvem: false, senhas: false, voz: false }, manha: true,
  descobertas: { falou: true, pediuAjuda: false, achouNota: false, trocouAtalho: false },
  ias: [], githubConfigurado: true, githubLogin: null, brainRepo: null, movimento: 1, sincronizar: null, googleConectado: false, vozQualquerCampo: true,
};
const params = new URLSearchParams(globalThis.location?.search ?? "");
let atual: EstadoTela = { ...base, tela: (params.get("tela") as EstadoTela["tela"]) ?? "passo1" };
let esperas = 0;

export const nada = async () => undefined;
export const estado = async () => atual;
export const tenancyIniciar = async (): Promise<Codigo> => {
  await new Promise((r) => setTimeout(r, 400));
  return { codigo: "ABCD-EFGH", url: "https://tenancy.exemplo/os/entrar", intervalo: 5 };
};
export const tenancyEsperar = async (): Promise<Espera> => {
  esperas += 1;
  if (esperas < 2) return "pendente";
  atual = { ...atual, tela: "passo2", passosFeitos: 1, antesDoPasso1: false, pessoa: { firstName: "Luan", name: "Luan Exemplo", email: "luan@exemplo.com" } };
  return "pronto";
};
export const iaConcluir = async () => { atual = { ...atual, tela: "passo3", passosFeitos: 2 }; };
export const githubIniciar = async (): Promise<Codigo> => ({ codigo: "4C2D-9KQX", url: "https://github.com/login/device", intervalo: 1 });
export const githubEsperar = async (): Promise<Espera> => { atual = { ...atual, githubLogin: "luan" }; return "pronto"; };
export const brainCriar = async (): Promise<OrigemBrain> => { atual = { ...atual, tela: "espacos", brainRepo: "luan-brain" }; return "modelo"; };
export const espacosTrazer = async () => { atual = { ...atual, tela: "fim", passosFeitos: 3 }; };
