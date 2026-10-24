import { useCallback, useEffect, useState } from "react";
import type { Codigo, ErroVisao, Espera } from "./tipos";

export type FaseCodigo =
  | { fase: "pedindo" }
  | { fase: "esperando"; codigo: Codigo }
  | { fase: "pronto"; codigo?: Codigo }
  | { fase: "erro"; erro: ErroVisao };

/** Pede o código, espera a aprovação no intervalo pedido e avisa quando termina. */
export function usarCodigo(iniciar: () => Promise<Codigo>, esperar: () => Promise<Espera>, aoTerminar: () => void, ligado = true) {
  const [f, setF] = useState<FaseCodigo>({ fase: "pedindo" });

  const pedir = useCallback(async () => {
    setF({ fase: "pedindo" });
    try {
      setF({ fase: "esperando", codigo: await iniciar() });
    } catch (e) {
      setF({ fase: "erro", erro: e as ErroVisao });
    }
  }, [iniciar]);

  useEffect(() => {
    if (ligado) void pedir();
  }, [ligado, pedir]);

  useEffect(() => {
    if (f.fase !== "esperando") return;
    let intervalo = f.codigo.intervalo * 1000;
    let parar = false;
    let t = 0;
    const passo = async () => {
      try {
        const r = await esperar();
        if (parar) return;
        if (r === "pronto") {
          setF({ fase: "pronto", codigo: f.codigo });
          aoTerminar();
          return;
        }
        if (r === "devagar") intervalo += 5000;
        t = window.setTimeout(passo, intervalo);
      } catch (e) {
        if (!parar) setF({ fase: "erro", erro: e as ErroVisao });
      }
    };
    t = window.setTimeout(passo, intervalo);
    return () => {
      parar = true;
      window.clearTimeout(t);
    };
  }, [f, esperar, aoTerminar]);

  return { f, pedir };
}
