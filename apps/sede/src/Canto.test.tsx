import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { Canto } from "./Canto";

describe("canto da atualização", () => {
  it("com os agentes parados, mostra Reiniciar", async () => {
    const reiniciar = vi.fn();
    render(<Canto versao="1.0.4" motivo={null} reiniciar={reiniciar} />);
    expect(screen.getByText("Atualização pronta · 1.0.4")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Reiniciar" }));
    expect(reiniciar).toHaveBeenCalled();
  });

  it("numa rodada, o motivo ocupa o lugar do botão", () => {
    render(<Canto versao="1.0.4" motivo="Os agentes estão numa rodada. O botão volta quando ela terminar." reiniciar={() => {}} />);
    expect(screen.queryByRole("button", { name: "Reiniciar" })).toBeNull();
    expect(screen.getByText("Os agentes estão numa rodada. O botão volta quando ela terminar.")).toBeInTheDocument();
  });
});
