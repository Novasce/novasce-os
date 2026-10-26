import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { BoasVindasTela, prazoCurto } from "./BoasVindas";
import * as ponte from "./ponte";
import { estadoDeTeste } from "./teste-dados";

vi.mock("./ponte");

const boas = { firstName: "Ana", area: { nome: "Comercial", dono: "Eduardo", url: "https://t/area" }, primeiraTarefa: { titulo: "conhecer o SOPEC por dentro", prazo: "2026-10-23T21:00:00Z", url: "https://t/tarefa" } };

describe("boas-vindas", () => {
  it("prazo vira dia da semana", () => {
    expect(prazoCurto("2026-10-23T15:00:00Z", new Date("2026-10-20T12:00:00Z"))).toBe("sexta");
    expect(prazoCurto("2026-12-01T15:00:00Z", new Date("2026-10-20T12:00:00Z"))).toBe("01/12");
  });

  it("antes dos três passos, com os três cartões", async () => {
    render(<BoasVindasTela estado={{ ...estadoDeTeste, tela: "boas-vindas", boasVindas: boas }} aoTerminar={() => {}} />);
    expect(screen.getByText("Boas-vindas à Novasce, Ana.")).toBeInTheDocument();
    expect(screen.getByText("Antes dos três passos, três coisas que ajudam no primeiro dia.")).toBeInTheDocument();
    expect(screen.getByText("Sua área: Comercial")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Abrir no Tenancy" }));
    expect(ponte.abrirUrl).toHaveBeenCalledWith("https://t/tarefa");
  });

  it("sem primeira tarefa, o cartão 3 não aparece; depois do passo 1 o texto muda", async () => {
    const fim = vi.fn();
    render(<BoasVindasTela estado={{ ...estadoDeTeste, tela: "boas-vindas", antesDoPasso1: false, boasVindas: { ...boas, primeiraTarefa: null } }} aoTerminar={fim} />);
    expect(screen.queryByText("Sua primeira tarefa")).toBeNull();
    expect(screen.getByText("Antes dos outros dois passos, duas coisas que ajudam no primeiro dia.")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Vamos para os passos" }));
    expect(ponte.boasVindasVista).toHaveBeenCalled();
  });
});
