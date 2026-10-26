import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { DeVolta } from "./DeVolta";
import * as ponte from "./ponte";
import { estadoDeTeste } from "./teste-dados";

vi.mock("./ponte");

const estado = {
  ...estadoDeTeste, tela: "de-volta" as const, passosFeitos: 1, pessoa: { firstName: "Luan", name: "Luan", email: "l@n.com" },
  brainRepo: "luan-brain", githubLogin: "luan", ias: ["claude"], googleConectado: true,
  espacos: [{ slug: "studio", nome: "Studio", descricao: "", estado: "trazido" as const }],
  sincronizar: { brain: { estado: "salvo" as const, quando: null, progresso: null, mensagem: null }, documentos: { estado: "subindo" as const, quando: null, progresso: 62, mensagem: null } },
};

describe("de volta", () => {
  it("o que mora na nuvem volta; o que é login, a pessoa refaz", async () => {
    render(<DeVolta estado={estado} recarregar={async () => {}} />);
    expect(screen.getByText("Que bom ter você de volta, Luan.")).toBeInTheDocument();
    expect(screen.getAllByText("de volta")).toHaveLength(2);
    expect(screen.getByText("62%")).toBeInTheDocument();
    expect(screen.getAllByText("entre de novo")).toHaveLength(2);
    await userEvent.click(screen.getByRole("button", { name: "Entrar no Claude" }));
    expect(ponte.iaAbrirWeb).toHaveBeenCalledWith("claude");
    await userEvent.click(screen.getByRole("button", { name: "Depois" }));
    expect(ponte.deVoltaConcluir).toHaveBeenCalled();
  });

  it("sem GitHub ainda, o brain pede a conexão na própria linha", () => {
    vi.mocked(ponte.githubIniciar).mockReturnValue(new Promise(() => {}));
    render(<DeVolta estado={{ ...estado, brainRepo: null, githubLogin: null }} recarregar={async () => {}} />);
    expect(screen.getByRole("button", { name: "Conectar o GitHub" })).toBeInTheDocument();
  });
});
