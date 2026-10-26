export type IdTela = "geral" | "comecar" | "arquivos" | "brain" | "navegador" | "loja" | "sala";

export type Ficha = {
  id: string;
  nome: string;
  oQueE: string;
  paraQueServe: string;
  comoFunciona: string;
  quantoCusta: string;
  precisaInstalar: string;
  /** "Mostrar o comando por trás". Nunca com token ou chave. */
  comando?: string;
};

export type Passo = { titulo: string; oQueFazer: string; ondeClicar: string; oQueDeveAparecer: string; exemplo?: string };
export type Problema = { oQueAparece: string; oQueFazer: string };
export type Guia = { id: string; titulo: string; rotuloNoGuia?: string; resumo: string; passos: Passo[]; problemas?: Problema[] };
export type Pergunta = { titulo: string; resposta: string };
export type AjudaDeTela = { tela: IdTela; nome: string; titulo: string; resumo: string; perguntas: Pergunta[] };
export type Recomendado = { id: string; nome: string; descricao: string; flatpak: string; ficha: string };
export type Palavra = { termo: string; definicao: string };
