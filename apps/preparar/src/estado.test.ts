import { describe, expect, it } from "vitest";
import { estadoInicial, formatarCodigo, lateral, reduzir, type Estado } from "./estado";

const naRede: Estado = { ...estadoInicial, passo: "rede", papel: "notebook" };
const noRegistro: Estado = { ...naRede, passo: "registro", rede: "Novasce Sede" };

describe("estado do Preparar", () => {
  it("papel definido leva à rede", () => {
    expect(reduzir(estadoInicial, { tipo: "papel-definido", papel: "sede" })).toMatchObject({ passo: "rede", papel: "sede" });
  });

  it("rede pronta leva ao registro", () => {
    expect(reduzir(naRede, { tipo: "rede-pronta", rede: "Novasce Sede" })).toMatchObject({ passo: "registro", rede: "Novasce Sede" });
  });

  it("o código aparece enquanto espera e a aprovação leva ao passo 4", () => {
    const esperando = reduzir(noRegistro, { tipo: "evento-do-registro", evento: { evento: "codigo", codigo: "R7M2PX", expiraEm: "x" } });
    expect(esperando.registro).toEqual({ tipo: "esperando", codigo: "R7M2PX" });
    const aprovado = reduzir(esperando, { tipo: "evento-do-registro", evento: { evento: "aprovado", maquina: "m1" } });
    expect(aprovado).toMatchObject({ passo: "dono", registro: { tipo: "aprovado" } });
  });

  it("sem o módulo os fica no registro até escolher Registrar depois", () => {
    const semModulo = reduzir(noRegistro, { tipo: "evento-do-registro", evento: { evento: "sem-modulo" } });
    expect(semModulo).toMatchObject({ passo: "registro", registro: { tipo: "sem-modulo" } });
    expect(reduzir(semModulo, { tipo: "registrar-depois" })).toMatchObject({ passo: "dono", registro: { tipo: "depois" } });
  });

  it("no notebook, a conta criada encerra; na sede, as contas se somam", () => {
    const noDono: Estado = { ...noRegistro, passo: "dono" };
    expect(reduzir(noDono, { tipo: "conta-criada", login: "luan" })).toMatchObject({ passo: "pronto", contas: ["luan"] });
    const sede: Estado = { ...noDono, papel: "sede" };
    const duas = reduzir(reduzir(sede, { tipo: "conta-criada", login: "luan" }), { tipo: "conta-criada", login: "ana" });
    expect(duas).toMatchObject({ passo: "dono", contas: ["luan", "ana"] });
    expect(reduzir(duas, { tipo: "concluir" }).passo).toBe("pronto");
  });

  it("a lateral repete o canvas: feito, agora e falta", () => {
    expect(lateral(noRegistro)).toEqual([
      { passo: "papel", texto: "✓ Papel: notebook", situacao: "feito" },
      { passo: "rede", texto: "✓ Rede: Novasce Sede", situacao: "feito" },
      { passo: "registro", texto: "3 · Registro", situacao: "agora" },
      { passo: "dono", texto: "4 · De quem é", situacao: "falta" },
    ]);
  });

  it("o código aparece em dois blocos", () => {
    expect(formatarCodigo("R7M2PX")).toBe("R7M 2PX");
    expect(formatarCodigo("ABC")).toBe("ABC");
  });
});
