import { describe, expect, it } from "vitest";
import { aplicarMovimento } from "./movimento";

describe("aplicarMovimento", () => {
  it("zero vira movimento reduzido", () => {
    const raiz = document.createElement("div");
    aplicarMovimento(0, raiz);
    expect(raiz.dataset.movimento).toBe("reduzido");
    expect(raiz.style.getPropertyValue("--fator")).toBe("0");
  });

  it("outro fator escala as durações e tira o reduzido", () => {
    const raiz = document.createElement("div");
    raiz.dataset.movimento = "reduzido";
    aplicarMovimento(0.5, raiz);
    expect(raiz.dataset.movimento).toBeUndefined();
    expect(raiz.style.getPropertyValue("--fator")).toBe("0.5");
  });

  it("valor estranho volta ao normal", () => {
    const raiz = document.createElement("div");
    aplicarMovimento(Number.NaN, raiz);
    expect(raiz.style.getPropertyValue("--fator")).toBe("1");
  });
});
