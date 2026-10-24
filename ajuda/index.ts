export * from "./tipos";
export { FICHAS, FICHAS_DA_CENTRAL } from "./conteudo/fichas";
export { GUIAS, GUIAS_DA_CENTRAL, SECOES_DO_GUIA } from "./conteudo/guias";
export { TELAS } from "./conteudo/telas";
export { RECOMENDADOS } from "./conteudo/recomendados";
export { PALAVRAS } from "./conteudo/palavras";
import { FICHAS } from "./conteudo/fichas";
import { GUIAS } from "./conteudo/guias";
import { TELAS } from "./conteudo/telas";
import type { AjudaDeTela, Ficha, Guia } from "./tipos";

export const fichaPorId = (id: string): Ficha | undefined => FICHAS.find((f) => f.id === id);
export const guiaPorId = (id: string): Guia | undefined => GUIAS.find((g) => g.id === id);
export const telaPorId = (id: string): AjudaDeTela => TELAS.find((t) => t.tela === id) ?? TELAS.find((t) => t.tela === "geral")!;
export { buscar, montarIndice, normalizar, type ItemIndice } from "./busca";
