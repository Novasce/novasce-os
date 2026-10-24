import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Painel } from "./Painel";

describe("painel da voz", () => {
  it("segurando: solte para escrever, com a barra de nível", () => {
    const { container } = render(<Painel estado={{ fase: "segurando", nivel: 0.6, mensagem: null }} reduzido={false} />);
    expect(screen.getByText("Ouvindo · solte para escrever")).toBeInTheDocument();
    expect(container.querySelectorAll(".barras i")).toHaveLength(5);
  });

  it("mãos livres e escrevendo dizem o que está acontecendo", () => {
    const { rerender } = render(<Painel estado={{ fase: "maos-livres", nivel: 0.2, mensagem: null }} reduzido={false} />);
    expect(screen.getByText("Ouvindo · toque de novo para escrever")).toBeInTheDocument();
    rerender(<Painel estado={{ fase: "escrevendo", nivel: 0, mensagem: null }} reduzido={false} />);
    expect(screen.getByText("Escrevendo o que você falou")).toBeInTheDocument();
  });

  it("menos movimento: barra parada e a palavra ouvindo", () => {
    const { container } = render(<Painel estado={{ fase: "segurando", nivel: 0.9, mensagem: null }} reduzido />);
    const alturas = [...container.querySelectorAll<HTMLElement>(".barras i")].map((i) => i.style.height);
    expect(new Set(alturas).size).toBe(1);
  });

  it("erro mostra a frase do motor", () => {
    render(<Painel estado={{ fase: "erro", nivel: 0, mensagem: "O texto está copiado. Cole com Ctrl+V." }} reduzido={false} />);
    expect(screen.getByText("O texto está copiado. Cole com Ctrl+V.")).toBeInTheDocument();
  });
});
