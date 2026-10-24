import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Passo2 } from "./Passo2";
import * as ponte from "./ponte";
import { estadoDeTeste } from "./teste-dados";

vi.mock("./ponte");

describe("passo 2", () => {
  it("só instala depois de marcar e diz o que está fazendo", async () => {
    let soltar: () => void = () => {};
    vi.mocked(ponte.iaInstalar).mockImplementation(() => new Promise<void>((r) => (soltar = r)));
    render(<Passo2 estado={estadoDeTeste} aoTerminar={() => {}} />);
    const instalar = screen.getByRole("button", { name: "Instalar e entrar" });
    expect(instalar).toHaveAttribute("aria-disabled", "true");
    await userEvent.click(screen.getByRole("button", { name: /^Claude/ }));
    await userEvent.click(instalar);
    expect(await screen.findByText("Instalando o Claude Code")).toBeInTheDocument();
    soltar();
    expect(await screen.findByText("Pronto. O login do Claude abriu no navegador.")).toBeInTheDocument();
    expect(ponte.iaAbrirWeb).toHaveBeenCalledWith("claude");
  });

  it("ainda não tenho IA pula o passo", async () => {
    const fim = vi.fn();
    render(<Passo2 estado={estadoDeTeste} aoTerminar={fim} />);
    await userEvent.click(screen.getByRole("button", { name: "Ainda não tenho IA" }));
    expect(ponte.iaConcluir).toHaveBeenCalledWith([]);
  });

  it("erro de instalação diz o que não aconteceu", async () => {
    vi.mocked(ponte.iaInstalar).mockRejectedValue({ tipo: "recusado", mensagem: "O Codex não foi instalado. Nada mudou no computador." });
    render(<Passo2 estado={estadoDeTeste} aoTerminar={() => {}} />);
    await userEvent.click(screen.getByRole("button", { name: /^ChatGPT/ }));
    await userEvent.click(screen.getByRole("button", { name: "Instalar e entrar" }));
    expect(await screen.findByText("O Codex não foi instalado. Nada mudou no computador.")).toBeInTheDocument();
  });
});
