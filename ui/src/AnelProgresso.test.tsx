import { readFileSync } from "node:fs";
import { join } from "node:path";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AnelProgresso } from "./AnelProgresso";

describe("AnelProgresso", () => {
  it("pinta um terço do contorno por passo", () => {
    const { container } = render(<AnelProgresso passos={2} />);
    const tercos = [...container.querySelectorAll("circle")].map((c) => c.getAttribute("data-feito"));
    expect(tercos).toEqual(["sim", "sim", "nao"]);
    expect(screen.getByRole("img", { name: "2 de 3 passos" })).toBeInTheDocument();
  });

  it("o cata-vento fica inteiro em qualquer passo", () => {
    for (const passos of [0, 1, 2, 3] as const) {
      const { container, unmount } = render(<AnelProgresso passos={passos} />);
      expect(container.querySelectorAll("path")).toHaveLength(3);
      unmount();
    }
  });

  it("a cor do terço troca em 240 ms e some com menos movimento", () => {
    const css = readFileSync(join(import.meta.dirname, "./anel.css"), "utf8");
    expect(css).toContain("transition: stroke var(--nv-passo) var(--nv-curva)");
    const tokens = readFileSync(join(import.meta.dirname, "./tokens.css"), "utf8");
    expect(tokens).toMatch(/--nv-passo: 0ms/);
  });
});
