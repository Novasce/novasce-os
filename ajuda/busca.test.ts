import { describe, expect, it } from "vitest";
import { buscar, montarIndice, normalizar } from "./busca";

describe("busca da ajuda", () => {
  it("ignora acento e maiúscula", () => {
    expect(normalizar("Começar Já")).toBe("comecar ja");
  });

  it("acha pela palavra em qualquer ordem e põe título na frente", () => {
    expect(buscar("senha")[0].titulo).toBe("Senhas e segurança");
    expect(buscar("senha esqueci")[0]).toMatchObject({ tipo: "pergunta", titulo: "Esqueci a senha de um site" });
  });

  it("acha ficha pelo nome", () => {
    expect(buscar("obsidian")[0]).toMatchObject({ tipo: "ficha", id: "obsidian" });
  });

  it("termo vazio não traz nada e o índice cobre tudo", () => {
    expect(buscar("   ")).toEqual([]);
    expect(montarIndice().length).toBeGreaterThan(30);
  });
});
