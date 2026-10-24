import { readFileSync } from "node:fs";
import { join } from "node:path";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Campo } from "./Campo";
import { propriedadesProibidasEmEstados } from "./test-utils/estados";

describe("Campo", () => {
  it("o rótulo nomeia a entrada", () => {
    render(<Campo rotulo="Senha do Wi-Fi" type="password" />);
    expect(screen.getByLabelText("Senha do Wi-Fi")).toHaveAttribute("type", "password");
  });

  it("a área da mensagem existe mesmo vazia (o erro não empurra a tela)", () => {
    const { container } = render(<Campo rotulo="Nome completo" />);
    const mensagem = container.querySelector(".nv-campo__mensagem");
    expect(mensagem).not.toBeNull();
    expect(mensagem?.textContent).toBe("");
    expect(screen.getByLabelText("Nome completo")).toHaveAttribute("aria-describedby", mensagem?.id);
  });

  it("com erro, marca a entrada e mostra a mensagem", () => {
    render(<Campo rotulo="E-mail" erro mensagem="Confira o e-mail." />);
    expect(screen.getByLabelText("E-mail")).toHaveAttribute("aria-invalid", "true");
    expect(screen.getByText("Confira o e-mail.")).toHaveClass("nv-campo__mensagem--erro");
  });

  it("foco e erro não mexem na geometria", () => {
    const css = readFileSync(join(import.meta.dirname, "./campo.css"), "utf8");
    expect(propriedadesProibidasEmEstados(css)).toEqual([]);
  });
});
