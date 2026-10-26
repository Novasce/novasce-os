import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Fim } from "./Fim";
import * as ponte from "./ponte";
import { estadoDeTeste } from "./teste-dados";

vi.mock("./ponte");

const estado = { ...estadoDeTeste, tela: "fim" as const, passosFeitos: 3, descobertas: { falou: true, pediuAjuda: false, achouNota: false, trocouAtalho: false } };

describe("fim do Começar", () => {
  it("textos do canvas e fim visto", () => {
    render(<Fim estado={estado} recarregar={async () => {}} />);
    expect(screen.getByText("Pronto. Os três passos estão feitos.")).toBeInTheDocument();
    expect(screen.getByText("Deixe o dia ainda mais fácil")).toBeInTheDocument();
    expect(screen.getByText("sem pontos e sem sequência de dias")).toBeInTheDocument();
    expect(screen.getByText(/Falou pela primeira vez/)).toBeInTheDocument();
    expect(ponte.fimVisto).toHaveBeenCalledTimes(1);
  });

  it("Meu dia vem marcado e desliga", async () => {
    render(<Fim estado={estado} recarregar={async () => {}} />);
    const caixa = screen.getByRole("checkbox", { name: /Abrir o Meu dia ao chegar de manhã/ });
    expect(caixa).toBeChecked();
    await userEvent.click(caixa);
    expect(ponte.manhaDefinir).toHaveBeenCalledWith(false);
  });

  it("extras chamam o que prometem", async () => {
    render(<Fim estado={estado} recarregar={async () => {}} />);
    await userEvent.click(screen.getByRole("button", { name: "Testar agora" }));
    expect(ponte.vozTestar).toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Abrir o Bitwarden" }));
    expect(ponte.abrirBitwarden).toHaveBeenCalled();
  });
});
