import { describe, expect, it } from "vitest";
import { gerarHtml, recusarTravessao } from "./html";

describe("Guia do Novasce OS", () => {
  const html = gerarHtml({ versao: "v1", data: "23/10/2026", catavento: "<svg></svg>" });

  it("tem a régua e o exemplo marcado", () => {
    for (const t of ["Guia do Novasce OS", "v1 · 23/10/2026 · Material do time", "O que fazer", "Onde clicar", "O que deve aparecer", "Exemplo · dados fictícios", "Palavras deste guia", "Neste guia"]) {
      expect(html).toContain(t);
    }
  });

  it("segue a ordem das seções do canvas", () => {
    const ordem = ["Primeiro dia", "A rotina", "Com o Tenancy", "Pedir ajuda", "Segurança", "Problemas comuns", "Por onde começar"].map((t) => html.indexOf(`<h2 id=`, html.indexOf(`>${t}</a>`)));
    expect(ordem.every((n) => n > 0)).toBe(true);
    expect(html.indexOf(">Problemas comuns</h2>")).toBeGreaterThan(html.indexOf(">Primeiro dia</h2>"));
  });

  it("problemas comuns em duas colunas e rodapé da casa", () => {
    expect(html).toContain("<th>O que aparece</th><th>O que fazer</th>");
    expect(html).toContain("Algo não bateu? Pedir ajuda, dizendo a seção. Versão publicada não muda: correção vira versão nova.");
  });

  it("tema claro fixo, sem escuro", () => {
    expect(html).not.toContain("prefers-color-scheme: dark");
  });

  it("recusa travessão", () => {
    expect(() => recusarTravessao("a \u2014 b")).toThrow(/travessão/);
    expect(html.includes("\u2014")).toBe(false);
  });
});
