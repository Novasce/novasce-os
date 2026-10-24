import { FICHAS } from "./conteudo/fichas";
import { GUIAS } from "./conteudo/guias";
import { TELAS } from "./conteudo/telas";

export type ItemIndice = { tipo: "guia" | "ficha" | "tela" | "pergunta"; id: string; titulo: string; trecho: string; texto: string };

export function normalizar(s: string): string {
  return s.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase().replace(/\s+/g, " ").trim();
}

export function montarIndice(): ItemIndice[] {
  const itens: ItemIndice[] = [];
  for (const g of GUIAS) {
    const corpo = [g.resumo, ...g.passos.flatMap((p) => [p.titulo, p.oQueFazer, p.ondeClicar, p.oQueDeveAparecer]), ...(g.problemas ?? []).flatMap((p) => [p.oQueAparece, p.oQueFazer])].join(" ");
    itens.push({ tipo: "guia", id: g.id, titulo: g.titulo, trecho: g.resumo, texto: normalizar(`${g.titulo} ${corpo}`) });
  }
  for (const f of FICHAS) {
    itens.push({ tipo: "ficha", id: f.id, titulo: f.nome, trecho: f.oQueE, texto: normalizar([f.nome, f.oQueE, f.paraQueServe, f.comoFunciona].join(" ")) });
  }
  for (const t of TELAS) {
    itens.push({ tipo: "tela", id: t.tela, titulo: t.nome, trecho: t.titulo, texto: normalizar(`${t.nome} ${t.titulo} ${t.resumo}`) });
    t.perguntas.forEach((p, i) => itens.push({ tipo: "pergunta", id: `${t.tela}#${i}`, titulo: p.titulo, trecho: p.resposta, texto: normalizar(`${p.titulo} ${p.resposta}`) }));
  }
  return itens;
}

const INDICE = montarIndice();

export function buscar(termo: string, limite = 8): ItemIndice[] {
  const palavras = normalizar(termo).split(" ").filter(Boolean);
  if (palavras.length === 0) return [];
  return INDICE.filter((i) => palavras.every((p) => i.texto.includes(p)))
    .map((i) => ({ i, nota: palavras.filter((p) => normalizar(i.titulo).includes(p)).length * 10 + (i.tipo === "guia" ? 2 : 0) }))
    .sort((a, b) => b.nota - a.nota)
    .slice(0, limite)
    .map((x) => x.i);
}
