import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import * as ponte from "./ponte";
import { estadoDeTeste } from "./teste-dados";

vi.mock("./ponte");

describe("App", () => {
  beforeEach(() => {
    vi.mocked(ponte.tenancyIniciar).mockResolvedValue({ codigo: "ABCD-EFGH", url: "https://t/os/entrar", intervalo: 5 });
    vi.mocked(ponte.tenancyEsperar).mockResolvedValue("pendente");
  });

  it("abre no passo pendente", async () => {
    vi.mocked(ponte.estado).mockResolvedValue({ ...estadoDeTeste, tela: "passo1" });
    render(<App />);
    expect(await screen.findByText("Passo 1 · Tenancy")).toBeInTheDocument();
  });

  it("aplica menos movimento vindo do KDE", async () => {
    vi.mocked(ponte.estado).mockResolvedValue({ ...estadoDeTeste, movimento: 0 });
    render(<App />);
    await screen.findByText("Passo 1 · Tenancy");
    expect(document.documentElement.dataset.movimento).toBe("reduzido");
  });

  it("erro ao carregar diz o que não aconteceu", async () => {
    vi.mocked(ponte.estado).mockRejectedValue({ tipo: "sem-servico", mensagem: "O serviço da máquina não respondeu." });
    render(<App />);
    expect(await screen.findByText("O Começar não carregou. Nada do que você já fez se perdeu.")).toBeInTheDocument();
  });
});
