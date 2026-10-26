import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { Ficha } from "./Ficha";

const f = { oQueE: "a", paraQueServe: "b", comoFunciona: "c", quantoCusta: "d", precisaInstalar: "e", comando: "git clone x" };

describe("Ficha", () => {
  it("tem os cinco campos da casa", () => {
    render(<Ficha {...f} />);
    for (const r of ["O que é", "Para que serve", "Como funciona", "Quanto custa", "Precisa instalar?"]) expect(screen.getByText(r)).toBeInTheDocument();
  });

  it("o comando por trás troca o conteúdo e volta", async () => {
    render(<Ficha {...f} />);
    await userEvent.click(screen.getByRole("button", { name: "Mostrar o comando por trás" }));
    expect(screen.getByText("git clone x")).toBeInTheDocument();
    expect(screen.queryByText("O que é")).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "Mostrar a ficha" }));
    expect(screen.getByText("O que é")).toBeInTheDocument();
  });

  it("ao trocar, a caixa guarda a altura da ficha e não encolhe", async () => {
    const { container } = render(<Ficha {...f} />);
    const caixa = container.querySelector<HTMLElement>(".nv-ficha")!;
    caixa.getBoundingClientRect = () => ({ height: 184.84375 }) as DOMRect;
    expect(caixa.style.minHeight).toBe("");
    await userEvent.click(screen.getByRole("button", { name: "Mostrar o comando por trás" }));
    expect(caixa.style.minHeight).toBe("184.84375px");
    await userEvent.click(screen.getByRole("button", { name: "Mostrar a ficha" }));
    expect(caixa.style.minHeight).toBe("184.84375px");
  });
});
