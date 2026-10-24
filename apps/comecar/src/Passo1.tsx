import { fichaPorId } from "@novasce/ajuda";
import { Botao, Ficha } from "@novasce/ui";
import { SemInternet } from "./Estados";
import * as ponte from "./ponte";
import { StatusCodigo } from "./Status";
import { usarCodigo } from "./usarCodigo";

export function Passo1({ aoTerminar }: { aoTerminar: () => void }) {
  const { f, pedir } = usarCodigo(ponte.tenancyIniciar, ponte.tenancyEsperar, aoTerminar);
  const ficha = fichaPorId("tenancy")!;
  if (f.fase === "erro" && f.erro.tipo === "sem-internet") return <SemInternet passo={1} />;
  const codigo = f.fase === "esperando" ? f.codigo : null;
  return (
    <>
      <span className="etq teal">Passo 1 · Tenancy</span>
      <h1 className="titulo-passo">Entre no Tenancy com o seu e-mail da Novasce.</h1>
      <div className="cartao sel">
        <b>Entrar no Tenancy</b>
        <span>Abrimos o Tenancy no navegador. Confira o código e aprove. Sem digitar senha aqui.</span>
        <div className="linha-codigo">
          <span className="mono codigo" aria-live="polite">{codigo ? codigo.codigo : <span className="esqueleto" style={{ display: "inline-block", width: "9ch", height: 30 }} />}</span>
          <Botao variante="primario" desligado={codigo ? undefined : "O código ainda está chegando."} onClick={() => codigo && void ponte.abrirUrl(codigo.url)}>
            Entrar no Tenancy
          </Botao>
          <Botao desligado={codigo ? undefined : "O código ainda está chegando."} onClick={() => codigo && void ponte.copiar(codigo.codigo)}>Copiar</Botao>
        </div>
        <StatusCodigo f={f} pedirOutro={() => void pedir()} />
      </div>
      <Ficha oQueE={ficha.oQueE} paraQueServe={ficha.paraQueServe} comoFunciona={ficha.comoFunciona} quantoCusta={ficha.quantoCusta} precisaInstalar={ficha.precisaInstalar} comando={ficha.comando} />
    </>
  );
}
