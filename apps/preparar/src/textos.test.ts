import { describe, expect, it } from "vitest";
import { textoDaPlaca } from "./passos/PassoPapel";
import { lerRegistro } from "./passos/PassoRegistro";

describe("textos das telas", () => {
  it("a placa NVIDIA na versão com NVIDIA, como no canvas", () => {
    expect(textoDaPlaca({ nvidia: "NVIDIA GeForce RTX 2060", varianteDaImagem: "nvidia" })).toEqual({
      texto: "Placa de vídeo encontrada: NVIDIA GeForce RTX 2060. Esta máquina usa a versão com NVIDIA.",
      tom: "ok",
    });
  });

  it("sem NVIDIA na versão padrão", () => {
    expect(textoDaPlaca({ nvidia: null, varianteDaImagem: "padrao" }).tom).toBe("ok");
  });

  it("versão errada para a placa vira alerta", () => {
    expect(textoDaPlaca({ nvidia: "NVIDIA GeForce RTX 2060", varianteDaImagem: "padrao" }).tom).toBe("alerta");
    expect(textoDaPlaca({ nvidia: null, varianteDaImagem: "nvidia" }).tom).toBe("alerta");
  });

  it("enquanto procura a placa, diz isso", () => {
    expect(textoDaPlaca(null)).toEqual({ texto: "Procurando a placa de vídeo.", tom: "neutro" });
  });

  it("o registro esperando mostra o código e a validade", () => {
    expect(lerRegistro({ tipo: "esperando", codigo: "R7M2PX" })).toMatchObject({
      codigo: "R7M 2PX",
      linha: "Esperando aprovação · o código vale 10 minutos",
      tom: "alerta",
    });
  });

  it("sem o módulo os oferece Registrar depois", () => {
    expect(lerRegistro({ tipo: "sem-modulo" }).acoes).toEqual(["depois", "tentar"]);
  });

  it("recusado (403) e vencido ou já usado (410) dizem o que houve e oferecem outro código", () => {
    expect(lerRegistro({ tipo: "negado" })).toMatchObject({
      linha: "O Tenancy recusou este código. Gere outro e peça a aprovação de novo.",
      acoes: ["outro-codigo"],
    });
    expect(lerRegistro({ tipo: "expirado" })).toMatchObject({
      linha: "O código venceu ou já foi usado. Gere outro código.",
      acoes: ["outro-codigo"],
    });
  });

  it("nenhum texto de tela tem travessão", () => {
    const leituras = (["parado", "aprovado", "negado", "expirado", "sem-modulo", "depois"] as const).map((tipo) => lerRegistro({ tipo }));
    for (const leitura of leituras) expect(leitura.linha).not.toContain("\u2014");
  });
});
