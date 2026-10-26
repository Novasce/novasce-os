import { useEffect, useRef, useState } from "react";
import { Botao } from "@novasce/ui";
import { api } from "../api";
import type { Papel } from "../estado";

export function PassoPronto({ papel, registrado }: { papel: Papel; registrado: boolean }) {
  const [erro, setErro] = useState("");
  const marcou = useRef(false);

  useEffect(() => {
    if (marcou.current) return;
    marcou.current = true;
    api.concluir().catch((e) => setErro(String(e)));
  }, []);

  return (
    <section className="passo">
      <h1>Máquina pronta</h1>
      <p className="texto">
        {papel === "sede"
          ? "Reinicie. A sede entra sozinha na conta Sede e abre a Sala em tela cheia."
          : "Reinicie para a pessoa entrar. Ela cria a própria senha no primeiro login."}
      </p>
      {!registrado && (
        <p className="texto texto--apagado">
          O registro no Tenancy ficou para depois. Quando o módulo estiver no ar, registre pelo terminal com sudo
          novasce-sinal pair.
        </p>
      )}
      <p className="aviso-erro" role="alert">{erro}</p>
      <div className="acoes">
        <Botao variante="primario" onClick={() => api.reiniciar().catch((e) => setErro(String(e)))}>
          Reiniciar agora
        </Botao>
        <Botao onClick={() => void api.fechar()}>Depois</Botao>
      </div>
    </section>
  );
}
