import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import * as ponte from "./ponte";
import { Recomendados } from "./Recomendados";

vi.mock("./ponte");

describe("Recomendados da casa", () => {
  it("já instalado aparece desligado e instalar chama o Flatpak por pessoa", async () => {
    vi.mocked(ponte.recomendadosInstalados).mockResolvedValue(["org.libreoffice.LibreOffice"]);
    vi.mocked(ponte.instalarRecomendado).mockResolvedValue();
    render(<Recomendados voltar={() => {}} abrirFicha={() => {}} />);
    expect(await screen.findByRole("button", { name: "Já instalado" })).toHaveAttribute("aria-disabled", "true");
    await userEvent.click(screen.getAllByRole("button", { name: "Instalar" })[0]);
    expect(ponte.instalarRecomendado).toHaveBeenCalledWith("us.zoom.Zoom");
    expect(await screen.findByText("Instalado só para você.")).toBeInTheDocument();
  });

  it("erro diz que nada mudou", async () => {
    vi.mocked(ponte.recomendadosInstalados).mockResolvedValue([]);
    vi.mocked(ponte.instalarRecomendado).mockRejectedValue("O app não foi instalado. Nada mudou no computador.");
    render(<Recomendados voltar={() => {}} abrirFicha={() => {}} />);
    await userEvent.click((await screen.findAllByRole("button", { name: "Instalar" }))[1]);
    expect(await screen.findByText("O app não foi instalado. Nada mudou no computador.")).toBeInTheDocument();
  });
});
