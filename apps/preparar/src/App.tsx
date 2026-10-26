import { useReducer } from "react";
import { Catavento } from "@novasce/ui";
import { estadoInicial, lateral, reduzir } from "./estado";
import { PassoDono } from "./passos/PassoDono";
import { PassoPapel } from "./passos/PassoPapel";
import { PassoPronto } from "./passos/PassoPronto";
import { PassoRede } from "./passos/PassoRede";
import { PassoRegistro } from "./passos/PassoRegistro";

export function App() {
  const [estado, despachar] = useReducer(reduzir, estadoInicial);
  const registrado = estado.registro.tipo === "aprovado";
  return (
    <div className="preparar">
      <aside className="lateral">
        <Catavento tamanho={60} cor="var(--nv-branco)" />
        <p className="lateral__titulo">Preparar esta máquina</p>
        <ol className="lateral__passos">
          {lateral(estado).map((item) => (
            <li
              key={item.passo}
              className={`lateral__passo lateral__passo--${item.situacao}`}
              aria-current={item.situacao === "agora" ? "step" : undefined}
            >
              {item.texto}
            </li>
          ))}
        </ol>
      </aside>
      <main className="conteudo">
        {estado.passo === "papel" && <PassoPapel aoDefinir={(papel) => despachar({ tipo: "papel-definido", papel })} />}
        {estado.passo === "rede" && <PassoRede aoConectar={(rede) => despachar({ tipo: "rede-pronta", rede })} />}
        {estado.passo === "registro" && <PassoRegistro registro={estado.registro} despachar={despachar} />}
        {estado.passo === "dono" && estado.papel && (
          <PassoDono papel={estado.papel} registrado={registrado} contas={estado.contas} despachar={despachar} />
        )}
        {estado.passo === "pronto" && estado.papel && <PassoPronto papel={estado.papel} registrado={registrado} />}
      </main>
    </div>
  );
}
