import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Central } from "./Central";
import * as ponte from "./ponte";

vi.mock("./ponte");

describe("Central de ajuda", () => {
  beforeEach(() => {
    vi.mocked(ponte.novidades).mockResolvedValue([{ versao: "1.0.4", linha: "O Começar ficou mais curto e a impressora da sede aparece sozinha." }]);
    vi.mocked(ponte.recomendadosInstalados).mockResolvedValue([]);
  });

  it("Nesta tela segue o app aberto, com o texto do canvas", async () => {
    render(<Central inicial={{ modo: "central", tela: "comecar", detalhe: null }} />);
    expect(screen.getByText("Nesta tela · Começar")).toBeInTheDocument();
    expect(screen.getByText("O Começar liga a sua conta do Tenancy, a sua IA e o seu brain.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Primeiro dia" })).toBeInTheDocument();
    expect(await screen.findByText(/O Começar ficou mais curto/)).toBeInTheDocument();
  });

  it("guia abre a régua no lugar e volta", async () => {
    render(<Central inicial={{ modo: "central", tela: "geral", detalhe: null }} />);
    await userEvent.click(screen.getByRole("button", { name: "Primeiro dia" }));
    expect(screen.getAllByText("O que fazer").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Onde clicar").length).toBeGreaterThan(0);
    expect(screen.getAllByText("O que deve aparecer").length).toBeGreaterThan(0);
    await userEvent.click(screen.getByRole("button", { name: "Voltar" }));
    expect(screen.getByText("Guias")).toBeInTheDocument();
  });

  it("busca acha a ficha e abre os cinco campos", async () => {
    render(<Central inicial={{ modo: "central", tela: "geral", detalhe: null }} />);
    await userEvent.type(screen.getByLabelText("Buscar na ajuda"), "obsidian");
    // O botão "Obsidian" das Fichas segue na coluna da direita; o primeiro da página é o resultado da busca.
    const [resultado] = screen.getAllByRole("button", { name: /^Obsidian/ });
    await userEvent.click(resultado);
    expect(screen.getByText("Para que serve")).toBeInTheDocument();
    expect(screen.getByText("Precisa instalar?")).toBeInTheDocument();
  });

  it("Pedir ajuda abre a janela do pedido", async () => {
    render(<Central inicial={{ modo: "central", tela: "geral", detalhe: null }} />);
    await userEvent.click(screen.getByRole("button", { name: "Pedir ajuda" }));
    expect(ponte.abrirPedir).toHaveBeenCalled();
  });

  it("abre direto num detalhe pedido pela linha de comando", () => {
    render(<Central inicial={{ modo: "detalhe", tela: "geral", detalhe: "ficha:drive" }} />);
    expect(screen.getByText("Arquivos na nuvem")).toBeInTheDocument();
  });
});
