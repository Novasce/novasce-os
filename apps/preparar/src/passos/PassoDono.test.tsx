import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "../api";
import { PassoDono } from "./PassoDono";

vi.mock("../api", () => ({
  api: { listarTime: vi.fn(), criarConta: vi.fn(), prepararSede: vi.fn() },
}));

const falso = vi.mocked(api);

beforeEach(() => {
  vi.resetAllMocks();
  falso.listarTime.mockResolvedValue([{ id: "u1", name: "Luan Silva", email: "luan@novasce.com", role: "DEV" }]);
  falso.criarConta.mockResolvedValue("luan");
  falso.prepararSede.mockResolvedValue(undefined);
});

describe("De quem é", () => {
  it("no notebook, escolhe da lista, cria a conta e grava o dono", async () => {
    const despachar = vi.fn();
    const pessoa = userEvent.setup();
    render(<PassoDono papel="notebook" registrado contas={[]} despachar={despachar} />);
    await pessoa.click(await screen.findByRole("radio", { name: /Luan Silva/ }));
    await pessoa.click(screen.getByRole("button", { name: "Criar a conta" }));
    expect(falso.criarConta).toHaveBeenCalledWith("Luan Silva", "luan@novasce.com", "u1");
    await waitFor(() => expect(despachar).toHaveBeenCalledWith({ tipo: "conta-criada", login: "luan" }));
  });

  it("na sede, a conta não tem dono e Concluir a sede prepara a conta Sede", async () => {
    const despachar = vi.fn();
    const pessoa = userEvent.setup();
    render(<PassoDono papel="sede" registrado contas={["ana"]} despachar={despachar} />);
    expect(screen.getByText("Contas criadas: ana.")).toBeInTheDocument();
    await pessoa.click(await screen.findByRole("radio", { name: /Luan Silva/ }));
    await pessoa.click(screen.getByRole("button", { name: "Criar a conta" }));
    expect(falso.criarConta).toHaveBeenCalledWith("Luan Silva", "luan@novasce.com", null);
    await waitFor(() => expect(despachar).toHaveBeenCalledWith({ tipo: "conta-criada", login: "luan" }));
    await pessoa.click(screen.getByRole("button", { name: "Concluir a sede" }));
    expect(falso.prepararSede).toHaveBeenCalled();
    await waitFor(() => expect(despachar).toHaveBeenCalledWith({ tipo: "concluir" }));
  });

  it("sem registro, pede nome e e-mail", () => {
    render(<PassoDono papel="notebook" registrado={false} contas={[]} despachar={vi.fn()} />);
    expect(screen.getByLabelText("Nome completo")).toBeInTheDocument();
    expect(falso.listarTime).not.toHaveBeenCalled();
  });
});
