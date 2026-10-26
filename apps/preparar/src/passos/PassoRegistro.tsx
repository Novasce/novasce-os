import { useCallback, useEffect, useRef } from "react";
import { Botao } from "@novasce/ui";
import { api } from "../api";
import { formatarCodigo, type Acao, type Registro } from "../estado";

export type Leitura = {
  codigo: string | null;
  linha: string;
  tom: "alerta" | "erro" | "neutro";
  acoes: Array<"outro-codigo" | "depois" | "tentar">;
};

export function lerRegistro(registro: Registro): Leitura {
  switch (registro.tipo) {
    case "parado":
      return { codigo: null, linha: "Pedindo um código ao Tenancy.", tom: "neutro", acoes: [] };
    case "esperando":
      return { codigo: formatarCodigo(registro.codigo), linha: "Esperando aprovação · o código vale 10 minutos", tom: "alerta", acoes: [] };
    case "aprovado":
      return { codigo: null, linha: "Aprovada no Tenancy.", tom: "neutro", acoes: [] };
    case "negado":
      return { codigo: null, linha: "O Tenancy recusou este código. Gere outro e peça a aprovação de novo.", tom: "erro", acoes: ["outro-codigo"] };
    case "expirado":
      return { codigo: null, linha: "O código venceu ou já foi usado. Gere outro código.", tom: "erro", acoes: ["outro-codigo"] };
    case "sem-modulo":
      return {
        codigo: null,
        linha: "O Tenancy ainda não recebe máquinas. Registre depois, quando o módulo estiver no ar. O resto segue agora.",
        tom: "alerta",
        acoes: ["depois", "tentar"],
      };
    case "erro":
      return { codigo: null, linha: `Não deu para falar com o Tenancy. ${registro.mensagem}`, tom: "erro", acoes: ["tentar", "depois"] };
    case "depois":
      return { codigo: null, linha: "O registro ficou para depois.", tom: "neutro", acoes: [] };
  }
}

export function PassoRegistro({ registro, despachar }: { registro: Registro; despachar: (acao: Acao) => void }) {
  const iniciou = useRef(false);

  const iniciar = useCallback(async () => {
    despachar({ tipo: "registro-iniciado" });
    try {
      await api.registrar();
    } catch (e) {
      despachar({ tipo: "evento-do-registro", evento: { evento: "erro", mensagem: String(e) } });
    }
  }, [despachar]);

  useEffect(() => {
    let ativo = true;
    let desligar: (() => void) | undefined;
    void api
      .aoRegistrar((evento) => despachar({ tipo: "evento-do-registro", evento }))
      .then((fn) => {
        if (!ativo) {
          fn();
          return;
        }
        desligar = fn;
        if (!iniciou.current) {
          iniciou.current = true;
          void iniciar();
        }
      });
    return () => {
      ativo = false;
      desligar?.();
    };
  }, [despachar, iniciar]);

  const leitura = lerRegistro(registro);
  return (
    <section className="passo">
      <h1>Registrar no Tenancy</h1>
      <p className="texto">
        No Tenancy, abra <b>Máquinas e telas</b> e aprove este código. A máquina só vai poder escrever o próprio estado:
        versão, atualização, disco e último sinal.
      </p>
      <p className="codigo" aria-live="polite">{leitura.codigo ?? " "}</p>
      <p className={`linha-estado linha-estado--solta linha-estado--${leitura.tom}`}>
        <span className="ponto" aria-hidden="true" />
        {leitura.linha}
      </p>
      <div className="acoes acoes--meio">
        {leitura.acoes.includes("depois") && (
          <Botao variante="primario" onClick={() => despachar({ tipo: "registrar-depois" })}>
            Registrar depois
          </Botao>
        )}
        {leitura.acoes.includes("outro-codigo") && (
          <Botao variante="primario" onClick={() => void iniciar()}>
            Gerar outro código
          </Botao>
        )}
        {leitura.acoes.includes("tentar") && <Botao onClick={() => void iniciar()}>Tentar de novo</Botao>}
      </div>
      <p className="nota">
        A senha do Wi-Fi digitada no passo 2 fica guardada só nesta máquina. Ela nunca vai para a imagem nem para o Tenancy.
      </p>
    </section>
  );
}
