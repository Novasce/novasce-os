import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { PedirAjuda } from "./PedirAjuda";
import * as ponte from "./ponte";

vi.mock("./ponte");

describe("Pedir ajuda", () => {
  beforeEach(() => {
    vi.mocked(ponte.prepararPedido).mockResolvedValue({ linhaMaquina: "Novasce OS 1.0.3 · notebook do Luan · atualizado há 2 dias · disco cifrado", miniatura: "data:image/png;base64,AA==" });
  });

  it("mostra o que vai junto e só envia com texto", async () => {
    render(<PedirAjuda />);
    expect(await screen.findByText(/Dados da máquina: Novasce OS 1.0.3/)).toBeInTheDocument();
    expect(screen.getByText("Nada além disso sai do computador.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Enviar para a Novasce" })).toHaveAttribute("aria-disabled", "true");
  });

  it("desmarcar o print manda sem print e fecha", async () => {
    vi.mocked(ponte.enviarPedido).mockResolvedValue({ tipo: "enviado", numero: "#128", mensagem: null });
    render(<PedirAjuda />);
    await screen.findByText(/Dados da máquina/);
    await userEvent.type(screen.getByLabelText("O que aconteceu?"), "A impressora da sede não aparece.");
    await userEvent.click(screen.getByRole("checkbox", { name: "Print desta tela" }));
    await userEvent.click(screen.getByRole("button", { name: "Enviar para a Novasce" }));
    expect(ponte.enviarPedido).toHaveBeenCalledWith("A impressora da sede não aparece.", false);
    expect(ponte.fecharJanela).toHaveBeenCalledWith("pedir");
  });

  it("sem internet: guardado, diz o que vai acontecer", async () => {
    vi.mocked(ponte.enviarPedido).mockResolvedValue({ tipo: "guardado", numero: null, mensagem: null });
    render(<PedirAjuda />);
    await screen.findByText(/Dados da máquina/);
    await userEvent.type(screen.getByLabelText("O que aconteceu?"), "Travou.");
    await userEvent.click(screen.getByRole("button", { name: "Enviar para a Novasce" }));
    expect(await screen.findByText("O chamado não foi enviado")).toBeInTheDocument();
    expect(screen.getByText("A internet caiu antes de chegar à Novasce. O pedido ficou guardado neste computador e sai sozinho quando a conexão voltar.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Tentar agora" })).toBeInTheDocument();
  });

  it("falar acrescenta o texto ditado", async () => {
    vi.mocked(ponte.ditarParar).mockResolvedValue("Não imprime.");
    render(<PedirAjuda />);
    await screen.findByText(/Dados da máquina/);
    await userEvent.click(screen.getByRole("button", { name: "Falar" }));
    await userEvent.click(screen.getByRole("button", { name: "Parar e escrever" }));
    expect(screen.getByLabelText("O que aconteceu?")).toHaveValue("Não imprime.");
  });
});
