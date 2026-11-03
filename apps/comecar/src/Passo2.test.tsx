import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { CAMINHO_DA_CHAVE, Passo2 } from "./Passo2";
import * as ponte from "./ponte";
import { estadoDeTeste } from "./teste-dados";

vi.mock("./ponte");

beforeEach(() => vi.resetAllMocks());

describe("passo 2", () => {
  it("só instala depois de marcar e diz o que está fazendo", async () => {
    let soltar: () => void = () => {};
    vi.mocked(ponte.iaInstalar).mockImplementation(() => new Promise<void>((r) => (soltar = r)));
    render(<Passo2 estado={estadoDeTeste} aoTerminar={() => {}} />);
    const instalar = screen.getByRole("button", { name: "Instalar e entrar" });
    expect(instalar).toHaveAttribute("aria-disabled", "true");
    await userEvent.click(screen.getByRole("button", { name: /^Claude/ }));
    expect(screen.getByText("Instala o Claude Code e abre o login no navegador.")).toBeInTheDocument();
    await userEvent.click(instalar);
    expect(await screen.findByText("Instalando o Claude Code")).toBeInTheDocument();
    soltar();
    expect(await screen.findByText("Pronto. O login do Claude abriu no navegador.")).toBeInTheDocument();
    expect(ponte.iaAbrirWeb).toHaveBeenCalledWith("claude");
  });

  it("com várias IAs, diz antes o que vai acontecer e conta o andamento", async () => {
    const soltas: Array<() => void> = [];
    vi.mocked(ponte.iaInstalar).mockImplementation(() => new Promise<void>((r) => soltas.push(r)));
    render(<Passo2 estado={estadoDeTeste} aoTerminar={() => {}} />);
    for (const nome of [/^Claude/, /^ChatGPT/, /^Gemini/]) await userEvent.click(screen.getByRole("button", { name: nome }));
    expect(screen.getByText("Instala uma por vez (Claude Code, Codex e Gemini CLI) e abre o login de cada uma no navegador.")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Instalar e entrar" }));
    expect(await screen.findByText("Instalando o Claude Code · 1 de 3")).toBeInTheDocument();
    soltas[0]();
    expect(await screen.findByText("Instalando o Codex · 2 de 3")).toBeInTheDocument();
    soltas[1]();
    expect(await screen.findByText("Instalando o Gemini CLI · 3 de 3")).toBeInTheDocument();
    soltas[2]();
    expect(await screen.findByText("Pronto. Os logins de Claude, ChatGPT e Gemini abriram no navegador.")).toBeInTheDocument();
  });

  it("ligar ao Tenancy diz por que espera, para que serve e qual IA liga", async () => {
    render(<Passo2 estado={estadoDeTeste} aoTerminar={() => {}} />);
    await userEvent.click(screen.getByRole("button", { name: /^ChatGPT/ }));
    await userEvent.click(screen.getByRole("button", { name: /^Claude/ }));
    await userEvent.type(screen.getByLabelText("Chave de IA do Tenancy"), "chave-de-teste");
    expect(screen.getByRole("button", { name: "Ligar" })).toHaveAttribute("aria-disabled", "true");
    expect(screen.getByText("Primeiro instale a IA, no botão Instalar e entrar.")).toBeInTheDocument();
    expect(screen.getByText(/passa a ver o seu quadro no Tenancy/)).toHaveTextContent("Liga uma IA: o Claude. O Codex ainda não liga por aqui.");
    await userEvent.click(screen.getByRole("button", { name: "Instalar e entrar" }));
    await screen.findByRole("button", { name: "Continuar" });
    const ligar = screen.getByRole("button", { name: "Ligar" });
    expect(ligar).not.toHaveAttribute("aria-disabled");
    await userEvent.click(ligar);
    expect(ponte.iaLigarTenancy).toHaveBeenCalledWith("claude", "chave-de-teste");
    expect(await screen.findByText("Ligada. O Claude já enxerga o seu Tenancy.")).toBeInTheDocument();
  });

  it("o link da chave abre a aba Chaves do Novasce AI Tools", async () => {
    render(<Passo2 estado={estadoDeTeste} aoTerminar={() => {}} />);
    await userEvent.click(screen.getByRole("button", { name: "Criar a chave no Tenancy" }));
    expect(CAMINHO_DA_CHAVE).toBe("/ia?tab=chaves");
    expect(ponte.abrirTenancy).toHaveBeenCalledWith("/ia?tab=chaves");
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
