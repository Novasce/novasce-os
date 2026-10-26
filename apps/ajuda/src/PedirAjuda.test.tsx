import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { PedirAjuda } from "./PedirAjuda";
import * as ponte from "./ponte";

vi.mock("./ponte");

describe("Pedir ajuda", () => {
  beforeEach(() => {
    vi.clearAllMocks();
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

  async function ateOGuardado() {
    vi.mocked(ponte.enviarPedido).mockResolvedValue({ tipo: "guardado", numero: null, mensagem: null });
    render(<PedirAjuda />);
    await screen.findByText(/Dados da máquina/);
    await userEvent.type(screen.getByLabelText("O que aconteceu?"), "Travou.");
    await userEvent.click(screen.getByRole("button", { name: "Enviar para a Novasce" }));
    await screen.findByText("O chamado não foi enviado");
  }

  const SEM_CONEXAO = "Ainda sem conexão. O pedido continua guardado e sai sozinho quando a internet voltar.";

  it("tentar agora sem conseguir sair: diz que continua guardado, na linha já reservada, e não fecha", async () => {
    vi.mocked(ponte.tentarAgora).mockResolvedValue(0);
    await ateOGuardado();
    const linha = document.querySelector(".status");
    expect(linha).not.toBeNull();
    expect(linha).toHaveTextContent("");
    expect(linha).toHaveAttribute("aria-live", "polite");
    await userEvent.click(screen.getByRole("button", { name: "Tentar agora" }));
    expect(await screen.findByText(SEM_CONEXAO)).toBe(linha);
    expect(ponte.fecharJanela).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Tentar agora" })).not.toHaveAttribute("aria-disabled");
  });

  it("tentar agora: enquanto espera o botão fica desligado e a mesma linha diz que está tentando", async () => {
    let terminar: (n: number) => void = () => {};
    vi.mocked(ponte.tentarAgora).mockReturnValue(new Promise<number>((r) => { terminar = r; }));
    await ateOGuardado();
    const linha = document.querySelector(".status");
    await userEvent.click(screen.getByRole("button", { name: "Tentar agora" }));
    expect(screen.getByRole("button", { name: "Tentar agora" })).toHaveAttribute("aria-disabled", "true");
    expect(linha).toHaveTextContent("Tentando enviar de novo");
    await userEvent.click(screen.getByRole("button", { name: "Tentar agora" }));
    expect(ponte.tentarAgora).toHaveBeenCalledTimes(1);
    terminar(0);
    await waitFor(() => expect(linha).toHaveTextContent(SEM_CONEXAO));
    expect(screen.getByRole("button", { name: "Tentar agora" })).not.toHaveAttribute("aria-disabled");
  });

  it("tentar agora de novo apaga o aviso antigo e, se o pedido sair, fecha a janela", async () => {
    vi.mocked(ponte.tentarAgora).mockResolvedValueOnce(0).mockResolvedValueOnce(1);
    await ateOGuardado();
    await userEvent.click(screen.getByRole("button", { name: "Tentar agora" }));
    await screen.findByText(SEM_CONEXAO);
    await userEvent.click(screen.getByRole("button", { name: "Tentar agora" }));
    await waitFor(() => expect(ponte.fecharJanela).toHaveBeenCalledWith("pedir"));
    expect(screen.queryByText(SEM_CONEXAO)).toBeNull();
  });

  it("tentar agora que falha por erro também avisa, sem fechar", async () => {
    vi.mocked(ponte.tentarAgora).mockRejectedValue(new Error("sem rede"));
    await ateOGuardado();
    await userEvent.click(screen.getByRole("button", { name: "Tentar agora" }));
    expect(await screen.findByText(SEM_CONEXAO)).toBeInTheDocument();
    expect(ponte.fecharJanela).not.toHaveBeenCalled();
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
