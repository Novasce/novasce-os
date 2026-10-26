import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { api } from "./api";
import type { EventoDoHelper } from "./estado";

vi.mock("./api", () => ({
  api: {
    placaDeVideo: vi.fn(),
    estadoDaRede: vi.fn(),
    redesWifi: vi.fn(),
    conectarWifi: vi.fn(),
    definirPapel: vi.fn(),
    registrar: vi.fn(),
    aoRegistrar: vi.fn(),
    listarTime: vi.fn(),
    criarConta: vi.fn(),
    prepararSede: vi.fn(),
    concluir: vi.fn(),
    reiniciar: vi.fn(),
    fechar: vi.fn(),
  },
}));

const falso = vi.mocked(api);

beforeEach(() => {
  vi.resetAllMocks();
  falso.placaDeVideo.mockResolvedValue({ nvidia: "NVIDIA GeForce RTX 2060", varianteDaImagem: "nvidia" });
  falso.definirPapel.mockResolvedValue(undefined);
  falso.estadoDaRede.mockResolvedValue({ conectada: true, porCabo: true });
  falso.redesWifi.mockResolvedValue([]);
  falso.aoRegistrar.mockResolvedValue(() => {});
  falso.registrar.mockResolvedValue(undefined);
});

describe("Preparar esta máquina", () => {
  it("o passo 1 mostra a placa e leva à rede", async () => {
    const pessoa = userEvent.setup();
    render(<App />);
    expect(
      await screen.findByText("Placa de vídeo encontrada: NVIDIA GeForce RTX 2060. Esta máquina usa a versão com NVIDIA."),
    ).toBeInTheDocument();
    await pessoa.click(screen.getByRole("radio", { name: "Sede" }));
    await pessoa.click(screen.getByRole("button", { name: "Continuar" }));
    expect(falso.definirPapel).toHaveBeenCalledWith("sede");
    expect(await screen.findByRole("heading", { name: "Conectar à internet" })).toBeInTheDocument();
  });

  it("sem o módulo os, registra depois e pede nome e e-mail", async () => {
    let ouvinte: ((evento: EventoDoHelper) => void) | undefined;
    falso.aoRegistrar.mockImplementation(async (cb) => {
      ouvinte = cb;
      return () => {};
    });
    falso.registrar.mockImplementation(async () => {
      ouvinte?.({ evento: "sem-modulo" });
    });
    const pessoa = userEvent.setup();
    render(<App />);
    await pessoa.click(await screen.findByRole("radio", { name: "Notebook" }));
    await pessoa.click(screen.getByRole("button", { name: "Continuar" }));
    await screen.findByText("Esta máquina já está na internet, pelo cabo.");
    await pessoa.click(screen.getByRole("button", { name: "Continuar" }));
    await pessoa.click(await screen.findByRole("button", { name: "Registrar depois" }));
    expect(await screen.findByRole("heading", { name: "De quem é este notebook?" })).toBeInTheDocument();
    expect(screen.getByLabelText("Nome completo")).toBeInTheDocument();
    expect(screen.getByLabelText("E-mail da Novasce")).toBeInTheDocument();
  });
});
