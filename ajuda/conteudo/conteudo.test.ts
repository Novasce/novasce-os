import { describe, expect, it } from "vitest";
import { FICHAS, FICHAS_DA_CENTRAL, GUIAS, GUIAS_DA_CENTRAL, PALAVRAS, RECOMENDADOS, SECOES_DO_GUIA, TELAS, fichaPorId, guiaPorId, telaPorId } from "../index";

describe("fonte única de ajuda", () => {
  it("nenhum texto tem travessão", () => {
    const tudo = JSON.stringify({ FICHAS, GUIAS, TELAS, RECOMENDADOS, PALAVRAS });
    expect(tudo.includes("\u2014")).toBe(false);
  });

  it("toda ficha tem os cinco campos", () => {
    for (const f of FICHAS) {
      for (const campo of [f.oQueE, f.paraQueServe, f.comoFunciona, f.quantoCusta, f.precisaInstalar]) {
        expect(campo.trim().length, `${f.id}`).toBeGreaterThan(3);
      }
    }
  });

  it("listas da Central e do Guia apontam para o que existe", () => {
    for (const id of [...GUIAS_DA_CENTRAL, ...SECOES_DO_GUIA]) expect(guiaPorId(id), id).toBeDefined();
    for (const id of FICHAS_DA_CENTRAL) expect(fichaPorId(id), id).toBeDefined();
    for (const r of RECOMENDADOS) expect(fichaPorId(r.ficha), r.id).toBeDefined();
  });

  it("todo passo tem a régua inteira", () => {
    for (const g of GUIAS) for (const p of g.passos) {
      expect(p.oQueFazer && p.ondeClicar && p.oQueDeveAparecer, `${g.id}: ${p.titulo}`).toBeTruthy();
    }
  });

  it("Nesta tela do Começar é o texto do canvas", () => {
    const t = telaPorId("comecar");
    expect(t.titulo).toBe("O Começar liga a sua conta do Tenancy, a sua IA e o seu brain.");
    expect(t.resumo).toBe("Leva uns 10 minutos. Dá para pular um passo e voltar depois: ele reabre onde parou.");
    expect(t.perguntas.map((p) => p.titulo)).toEqual(["O que cada passo faz", "Não tenho conta no GitHub", "Ainda não tenho IA"]);
  });

  it("tela desconhecida cai na geral", () => {
    expect(telaPorId("qualquer").tela).toBe("geral");
  });
});
