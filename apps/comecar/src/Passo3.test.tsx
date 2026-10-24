import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Passo3 } from "./Passo3";
import * as ponte from "./ponte";
import { estadoDeTeste } from "./teste-dados";

vi.mock("./ponte");

describe("passo 3", () => {
  beforeEach(() => vi.useFakeTimers({ shouldAdvanceTime: true }));
  afterEach(() => vi.useRealTimers());

  it("criar o brain fica desligado até o GitHub conectar", async () => {
    vi.mocked(ponte.githubIniciar).mockResolvedValue({ codigo: "4C2D-9KQX", url: "https://github.com/login/device", intervalo: 5 });
    vi.mocked(ponte.githubEsperar).mockResolvedValue("pronto");
    vi.mocked(ponte.brainCriar).mockResolvedValue("modelo");
    const fim = vi.fn();
    render(<Passo3 estado={{ ...estadoDeTeste, tela: "passo3" }} aoTerminar={fim} />);
    expect(await screen.findByText("4C2D-9KQX")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Criar meu brain" })).toHaveAttribute("aria-disabled", "true");
    await vi.advanceTimersByTimeAsync(5100);
    expect(await screen.findByText("Conectado")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Criar meu brain" }));
    expect(ponte.brainCriar).toHaveBeenCalled();
    expect(fim).toHaveBeenCalled();
  });

  it("sem OAuth App explica e oferece pular", async () => {
    render(<Passo3 estado={{ ...estadoDeTeste, tela: "passo3", githubConfigurado: false }} aoTerminar={() => {}} />);
    expect(screen.getByText("O GitHub ainda não foi ligado ao Novasce OS pela Novasce.")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Pular por agora" }));
    expect(ponte.fechar).toHaveBeenCalled();
  });
});
