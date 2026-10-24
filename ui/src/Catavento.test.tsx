import { readFileSync } from "node:fs";
import { join } from "node:path";
import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Catavento } from "./Catavento";

const oficial = readFileSync(join(import.meta.dirname, "../../marca/catavento.svg"), "utf8");
const pasOficiais = [...oficial.matchAll(/ d="([^"]+)"/g)].map((m) => m[1]);

describe("Catavento", () => {
  it("usa as três pás do vetor oficial, sem redesenho", () => {
    const { container } = render(<Catavento />);
    const pas = [...container.querySelectorAll("path")].map((p) => p.getAttribute("d"));
    expect(pasOficiais).toHaveLength(3);
    expect(pas).toEqual(pasOficiais);
  });

  it("é decorativo sem título e imagem com título", () => {
    const { container, rerender } = render(<Catavento />);
    expect(container.querySelector("svg")).toHaveAttribute("aria-hidden", "true");
    rerender(<Catavento titulo="Novasce" />);
    expect(container.querySelector("svg")).toHaveAttribute("aria-label", "Novasce");
  });
});
