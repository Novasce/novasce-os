import { describe, expect, it } from "vitest";
import { propriedadesProibidasEmEstados } from "./estados";

describe("regra da casa: estado não mexe na geometria", () => {
  it("aceita cor, fundo, borda, contorno e sombra", () => {
    const css = ".b:hover { background: red; border-color: blue; outline: 2px solid; box-shadow: 0 0 1px; color: #fff; }";
    expect(propriedadesProibidasEmEstados(css)).toEqual([]);
  });

  it("acusa padding, largura e borda que muda de espessura", () => {
    const css = ".b:focus-visible { padding: 4px; } .c[aria-checked=\"true\"] { border: 2px solid; width: 10px; }";
    expect(propriedadesProibidasEmEstados(css)).toEqual([
      ".b:focus-visible → padding",
      ".c[aria-checked=\"true\"] → border",
      ".c[aria-checked=\"true\"] → width",
    ]);
  });

  it("ignora regras sem estado", () => {
    expect(propriedadesProibidasEmEstados(".b { padding: 4px; }")).toEqual([]);
  });
});
