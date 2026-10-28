import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "../api";
import { lerSenha, PassoDono } from "./PassoDono";

vi.mock("../api", () => ({
  api: { listarTime: vi.fn(), criarConta: vi.fn(), prepararSede: vi.fn() },
}));

const falso = vi.mocked(api);
const SENHA = "cata vento 7";

async function digitarSenha(pessoa: ReturnType<typeof userEvent.setup>, senha = SENHA, repetida = senha) {
  await pessoa.type(screen.getByLabelText("Senha da pessoa"), senha);
  await pessoa.type(screen.getByLabelText("Repita a senha"), repetida);
}

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
    await digitarSenha(pessoa);
    await pessoa.click(screen.getByRole("button", { name: "Criar a conta" }));
    expect(falso.criarConta).toHaveBeenCalledWith("Luan Silva", "luan@novasce.com", SENHA, "u1");
    await waitFor(() => expect(despachar).toHaveBeenCalledWith({ tipo: "conta-criada", login: "luan" }));
  });

  it("na sede, a conta não tem dono e Concluir a sede prepara a conta Sede", async () => {
    const despachar = vi.fn();
    const pessoa = userEvent.setup();
    render(<PassoDono papel="sede" registrado contas={["ana"]} despachar={despachar} />);
    expect(screen.getByText("Contas criadas: ana.")).toBeInTheDocument();
    await pessoa.click(await screen.findByRole("radio", { name: /Luan Silva/ }));
    await digitarSenha(pessoa);
    await pessoa.click(screen.getByRole("button", { name: "Criar a conta" }));
    expect(falso.criarConta).toHaveBeenCalledWith("Luan Silva", "luan@novasce.com", SENHA, null);
    // Depois de criar, a senha some da tela.
    await waitFor(() => expect(screen.getByLabelText("Senha da pessoa")).toHaveValue(""));
    await waitFor(() => expect(despachar).toHaveBeenCalledWith({ tipo: "conta-criada", login: "luan" }));
    await pessoa.click(screen.getByRole("button", { name: "Concluir a sede" }));
    expect(falso.prepararSede).toHaveBeenCalled();
    await waitFor(() => expect(despachar).toHaveBeenCalledWith({ tipo: "concluir" }));
  });

  it("sem a senha, ou com as duas diferentes, não cria a conta", async () => {
    const pessoa = userEvent.setup();
    render(<PassoDono papel="sede" registrado contas={[]} despachar={vi.fn()} />);
    await pessoa.click(await screen.findByRole("radio", { name: /Luan Silva/ }));
    const criar = screen.getByRole("button", { name: "Criar a conta" });
    expect(criar).toBeDisabled();
    await digitarSenha(pessoa, SENHA, "outra senha");
    expect(criar).toBeDisabled();
    expect(screen.getByText("As duas senhas não batem.")).toBeInTheDocument();
    expect(screen.getByLabelText("Senha da pessoa")).toHaveAttribute("type", "password");
  });

  it("a senha pede o mínimo e as duas iguais", () => {
    expect(lerSenha("", "")).toEqual({ pronta: false, aviso: "" });
    expect(lerSenha("1234567", "1234567").pronta).toBe(false);
    expect(lerSenha("12345678", "12345678")).toEqual({ pronta: true, aviso: "" });
    expect(lerSenha("12345678", "1234")).toEqual({ pronta: false, aviso: "As duas senhas não batem." });
  });

  it("sem registro, pede nome e e-mail", () => {
    render(<PassoDono papel="notebook" registrado={false} contas={[]} despachar={vi.fn()} />);
    expect(screen.getByLabelText("Nome completo")).toBeInTheDocument();
    expect(falso.listarTime).not.toHaveBeenCalled();
  });
});
