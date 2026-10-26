import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Passo1 } from "./Passo1";
import * as ponte from "./ponte";

vi.mock("./ponte");

describe("passo 1", () => {
  beforeEach(() => vi.useFakeTimers({ shouldAdvanceTime: true }));
  afterEach(() => vi.useRealTimers());

  it("mostra o código, abre o Tenancy e espera", async () => {
    vi.mocked(ponte.tenancyIniciar).mockResolvedValue({ codigo: "ABCD-EFGH", url: "https://t/os/entrar", intervalo: 5 });
    vi.mocked(ponte.tenancyEsperar).mockResolvedValue("pendente");
    render(<Passo1 aoTerminar={() => {}} />);
    expect(await screen.findByText("ABCD-EFGH")).toBeInTheDocument();
    expect(screen.getByText("Esperando você aprovar no navegador")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Entrar no Tenancy" }));
    expect(ponte.abrirUrl).toHaveBeenCalledWith("https://t/os/entrar");
  });

  it("avisa quando aprovado", async () => {
    const fim = vi.fn();
    vi.mocked(ponte.tenancyIniciar).mockResolvedValue({ codigo: "ABCD-EFGH", url: "https://t/os/entrar", intervalo: 1 });
    vi.mocked(ponte.tenancyEsperar).mockResolvedValue("pronto");
    render(<Passo1 aoTerminar={fim} />);
    await screen.findByText("ABCD-EFGH");
    await vi.advanceTimersByTimeAsync(1100);
    expect(fim).toHaveBeenCalled();
  });

  it("sem internet mostra o estado da prancha 19", async () => {
    vi.mocked(ponte.tenancyIniciar).mockRejectedValue({ tipo: "sem-internet", mensagem: "Sem internet." });
    render(<Passo1 aoTerminar={() => {}} />);
    expect(await screen.findByText("O passo 1 precisa de internet")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Abrir o Wi-Fi" })).toBeInTheDocument();
  });
});
