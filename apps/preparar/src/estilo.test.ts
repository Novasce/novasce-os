import { readFileSync } from "node:fs";
import { join } from "node:path";
import { propriedadesProibidasEmEstados } from "@novasce/ui/teste";
import { describe, expect, it } from "vitest";

describe("estilo do Preparar", () => {
  it("seleção, hover e foco não mexem na geometria", () => {
    const css = readFileSync(join(import.meta.dirname, "./estilo.css"), "utf8");
    expect(propriedadesProibidasEmEstados(css)).toEqual([]);
  });
});
