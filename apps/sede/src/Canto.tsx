import { Botao, Catavento } from "@novasce/ui";

export function Canto({ versao, motivo, reiniciar }: { versao: string; motivo: string | null; reiniciar: () => void }) {
  return (
    <div className="canto">
      <span className="selo" aria-hidden="true"><Catavento tamanho={14} cor="#fff" /></span>
      <div className="canto-texto">
        <b>Atualização pronta · {versao}</b>
        <div className="canto-acao">{motivo ? <span>{motivo}</span> : <Botao tamanho="pequeno" variante="primario" onClick={reiniciar}>Reiniciar</Botao>}</div>
      </div>
    </div>
  );
}
