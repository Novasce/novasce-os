import { fichaPorId, guiaPorId, telaPorId } from "@novasce/ajuda";
import { Botao, Ficha } from "@novasce/ui";

/** "guia:primeiro-dia", "ficha:drive", "tela:comecar" ou "pergunta:comecar#0". */
export function Detalhe({ alvo, voltar }: { alvo: string; voltar: () => void }) {
  const [tipo, id] = alvo.split(":");
  let corpo: JSX.Element | null = null;
  if (tipo === "guia") {
    const g = guiaPorId(id);
    if (g)
      corpo = (
        <>
          <h2>{g.titulo}</h2>
          <p className="resumo">{g.resumo}</p>
          {g.passos.map((p) => (
            <div key={p.titulo} className="cartao">
              <b>{p.titulo}</b>
              <div className="regua">
                <div><span className="etq">O que fazer</span><p>{p.oQueFazer}</p></div>
                <div><span className="etq">Onde clicar</span><p>{p.ondeClicar}</p></div>
                <div><span className="etq">O que deve aparecer</span><p>{p.oQueDeveAparecer}</p></div>
              </div>
            </div>
          ))}
          {g.problemas?.length ? (
            <table className="problemas"><thead><tr><th>O que aparece</th><th>O que fazer</th></tr></thead>
              <tbody>{g.problemas.map((p) => <tr key={p.oQueAparece}><td>{p.oQueAparece}</td><td>{p.oQueFazer}</td></tr>)}</tbody></table>
          ) : null}
        </>
      );
  } else if (tipo === "ficha") {
    const f = fichaPorId(id);
    if (f) corpo = <><h2>{f.nome}</h2><Ficha oQueE={f.oQueE} paraQueServe={f.paraQueServe} comoFunciona={f.comoFunciona} quantoCusta={f.quantoCusta} precisaInstalar={f.precisaInstalar} comando={f.comando} /></>;
  } else if (tipo === "pergunta" || tipo === "tela") {
    const [tela, n] = id.split("#");
    const t = telaPorId(tela);
    const p = n !== undefined ? t.perguntas[Number(n)] : null;
    corpo = p ? <><h2>{p.titulo}</h2><p className="resumo">{p.resposta}</p></> : <><h2>{t.titulo}</h2><p className="resumo">{t.resumo}</p></>;
  }
  return (
    <section className="detalhe">
      <Botao tamanho="pequeno" onClick={voltar}>Voltar</Botao>
      {corpo ?? <p className="resumo">Este assunto não está mais na ajuda. Peça ajuda dizendo o que procurava.</p>}
    </section>
  );
}
