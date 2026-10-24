import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { cores, movimento } from "./tokens";

const css = readFileSync(join(import.meta.dirname, "tokens.css"), "utf8");

describe("tokens da marca", () => {
  it("o teal da casa é #00666C e o grafite é #1E1E20", () => {
    expect(cores.teal).toBe("#00666C");
    expect(cores.grafite).toBe("#1E1E20");
    expect(cores.ciano).toBe("#00C4CC");
  });

  it("cada cor do tokens.ts existe no tokens.css", () => {
    for (const valor of Object.values(cores)) {
      expect(css.toUpperCase()).toContain(valor.toUpperCase());
    }
  });

  it("o movimento segue a spec: uma curva, 160, 200 e 240 ms", () => {
    expect(movimento).toEqual({ curva: "cubic-bezier(.2, .7, .2, 1)", trocaMs: 160, entraMs: 200, passoMs: 240 });
    expect(css).toContain("--nv-curva: cubic-bezier(.2, .7, .2, 1)");
  });

  it("menos movimento zera as durações e deixa a cor", () => {
    expect(css).toContain("prefers-reduced-motion: reduce");
    expect(css).toMatch(/--nv-troca: 0ms/);
  });
});
