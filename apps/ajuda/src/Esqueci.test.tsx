import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Esqueci } from "./Esqueci";
import * as ponte from "./ponte";

vi.mock("./ponte");

describe("Esqueci a senha", () => {
  it("QR com os textos da prancha 15", async () => {
    vi.mocked(ponte.esqueci).mockResolvedValue({ svg: "<svg data-qr></svg>", titulo: "Esqueceu a senha, Luan?", linha: "Só o Eduardo troca a senha deste computador." });
    const { container } = render(<Esqueci voltar={() => {}} />);
    expect(await screen.findByText("Esqueceu a senha, Luan?")).toBeInTheDocument();
    expect(screen.getByText("Sem celular? Fale com o Eduardo.")).toBeInTheDocument();
    expect(container.querySelector("[data-qr]")).not.toBeNull();
  });

  it("máquina sem registro diz o que fazer", async () => {
    vi.mocked(ponte.esqueci).mockRejectedValue("Esta máquina ainda não foi registrada. Fale com o Eduardo.");
    render(<Esqueci voltar={() => {}} />);
    expect(await screen.findByText("Esta máquina ainda não foi registrada. Fale com o Eduardo.")).toBeInTheDocument();
  });
});
