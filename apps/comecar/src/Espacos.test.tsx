import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Espacos, rotuloTrazer } from "./Espacos";
import * as ponte from "./ponte";
import { estadoDeTeste } from "./teste-dados";

vi.mock("./ponte");

const estado = {
  ...estadoDeTeste, tela: "espacos" as const, passosFeitos: 2, brainRepo: "luan-brain",
  espacos: [
    { slug: "studio", nome: "Studio", descricao: "o institucional da Novasce · todo o time participa", estado: "trazer" as const },
    { slug: "food", nome: "Food", descricao: "não é do seu setor no Tenancy", estado: "fora" as const },
  ],
};

describe("espaços", () => {
  it("rótulo do botão", () => {
    expect(rotuloTrazer(["Studio"], false)).toBe("Trazer o Studio");
    expect(rotuloTrazer(["Studio", "Food"], true)).toBe("Trazer o Studio, o Food e o kit");
    expect(rotuloTrazer([], false)).toBe("Continuar");
  });

  it("traz só o marcado; Food fica fora com o motivo", async () => {
    const fim = vi.fn();
    render(<Espacos estado={estado} aoTerminar={fim} />);
    expect(screen.getByText("luan-brain")).toBeInTheDocument();
    expect(screen.getByText("não é do seu setor no Tenancy")).toBeInTheDocument();
    expect(screen.getByText("chega pelo Tenancy quando você tiver uma frente lá")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Trazer o Studio" }));
    expect(ponte.espacosTrazer).toHaveBeenCalledWith(["studio"], false);
    expect(fim).toHaveBeenCalled();
  });

  it("sem permissão diz o que não veio", async () => {
    vi.mocked(ponte.espacosTrazer).mockRejectedValue({ tipo: "sem-permissao", mensagem: "O Studio não veio. A sua conta do GitHub ainda não tem acesso a ele. Peça ajuda e a Novasce libera." });
    render(<Espacos estado={estado} aoTerminar={() => {}} />);
    await userEvent.click(screen.getByRole("button", { name: "Trazer o Studio" }));
    expect(await screen.findByText(/O Studio não veio/)).toBeInTheDocument();
  });
});
