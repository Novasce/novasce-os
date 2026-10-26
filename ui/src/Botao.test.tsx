import { readFileSync } from "node:fs";
import { join } from "node:path";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Botao } from "./Botao";
import { propriedadesProibidasEmEstados } from "./test-utils/estados";

describe("Botao", () => {
  it("é type=button e secundário por padrão", () => {
    render(<Botao>Depois</Botao>);
    const botao = screen.getByRole("button", { name: "Depois" });
    expect(botao).toHaveAttribute("type", "button");
    expect(botao).toHaveClass("nv-botao", "nv-botao--secundario");
  });

  it("primário chama o clique; desligado não", async () => {
    const clique = vi.fn();
    const pessoa = userEvent.setup();
    const { rerender } = render(<Botao variante="primario" onClick={clique}>Continuar</Botao>);
    expect(screen.getByRole("button")).toHaveClass("nv-botao--primario");
    await pessoa.click(screen.getByRole("button"));
    expect(clique).toHaveBeenCalledTimes(1);
    rerender(<Botao variante="primario" onClick={clique} disabled>Continuar</Botao>);
    await pessoa.click(screen.getByRole("button"));
    expect(clique).toHaveBeenCalledTimes(1);
  });

  it("hover, foco e desligado não mexem na geometria", () => {
    const css = readFileSync(join(import.meta.dirname, "./botao.css"), "utf8");
    expect(propriedadesProibidasEmEstados(css)).toEqual([]);
  });
});
