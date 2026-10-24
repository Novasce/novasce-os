import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { VoltaSala } from "./VoltaSala";

describe("volta à Sala", () => {
  beforeEach(() => vi.useFakeTimers({ shouldAdvanceTime: true }));
  afterEach(() => vi.useRealTimers());

  it("conta 30 s e volta", async () => {
    const voltar = vi.fn();
    render(<VoltaSala segundos={30} voltar={voltar} continuar={() => {}} />);
    expect(screen.getByText("Ninguém mexe há 10 minutos")).toBeInTheDocument();
    expect(screen.getByText("30 s")).toBeInTheDocument();
    expect(screen.getByText(/O que estava aberto no navegador será esquecido./)).toBeInTheDocument();
    await vi.advanceTimersByTimeAsync(30_000);
    expect(voltar).toHaveBeenCalledTimes(1);
  });

  it("continuar usando cancela", async () => {
    const voltar = vi.fn();
    const continuar = vi.fn();
    render(<VoltaSala segundos={30} voltar={voltar} continuar={continuar} />);
    await userEvent.click(screen.getByRole("button", { name: "Continuar usando" }));
    expect(continuar).toHaveBeenCalled();
  });
});
