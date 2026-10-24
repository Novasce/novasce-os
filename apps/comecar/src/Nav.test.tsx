import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Nav, resumoDosPassos } from "./Nav";
import { estadoDeTeste } from "./teste-dados";

describe("coluna dos passos", () => {
  it("diz quantos passos faltam como no canvas", () => {
    expect(resumoDosPassos(2, "passo3")).toEqual(["2 de 3 passos", "falta GitHub e brain"]);
    expect(resumoDosPassos(2, "espacos")).toEqual(["Quase 3 de 3", "faltam os espaços"]);
    expect(resumoDosPassos(0, "passo1")).toEqual(["0 de 3 passos", "leva uns 10 minutos"]);
  });

  it("marca feito, agora e falta, e mostra quem entrou", () => {
    render(<Nav estado={{ ...estadoDeTeste, tela: "passo3", passosFeitos: 2, pessoa: { firstName: "Luan", name: "Luan", email: "luan@novasce.com" } }} />);
    expect(screen.getByText("Tenancy").closest("span")).toHaveClass("feito");
    expect(screen.getByText("GitHub e brain").closest("span")).toHaveClass("agora");
    expect(screen.getByText("Luan")).toBeInTheDocument();
    expect(screen.getByText("Dá para pular e voltar. O Começar fica no dock.")).toBeInTheDocument();
  });
});
