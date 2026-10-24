import { Botao, Catavento } from "@novasce/ui";
import * as ponte from "./ponte";
import type { EstadoTela } from "./tipos";

const NUMEROS = ["", "uma coisa", "duas coisas", "três coisas"];

export function prazoCurto(iso: string, agora = new Date()): string {
  const d = new Date(iso);
  const dias = (d.getTime() - agora.getTime()) / 86_400_000;
  if (dias >= 0 && dias < 7) return new Intl.DateTimeFormat("pt-BR", { weekday: "long", timeZone: "America/Sao_Paulo" }).format(d).replace("-feira", "");
  return new Intl.DateTimeFormat("pt-BR", { day: "2-digit", month: "2-digit", timeZone: "America/Sao_Paulo" }).format(d);
}

export function BoasVindasTela({ estado, aoTerminar }: { estado: EstadoTela; aoTerminar: () => void }) {
  const b = estado.boasVindas!;
  const cartoes: { titulo: string; texto: string; botao: string; primario?: boolean; acao: () => void }[] = [
    { titulo: "A casa", texto: "O Manual da Novasce: como a gente trabalha, as áreas e quem cuida de cada uma.", botao: "Ler agora", primario: true, acao: () => void ponte.abrirTenancy("/time/material") },
  ];
  if (b.area) cartoes.push({ titulo: `Sua área: ${b.area.nome}`, texto: `Quem cuida: ${b.area.dono}. O documento da área abre aqui.`, botao: "Abrir", acao: () => void ponte.abrirUrl(b.area!.url) });
  if (b.primeiraTarefa)
    cartoes.push({
      titulo: "Sua primeira tarefa",
      texto: `Já está no Tenancy: ${b.primeiraTarefa.titulo}${b.primeiraTarefa.prazo ? ` · prazo ${prazoCurto(b.primeiraTarefa.prazo)}` : ""}.`,
      botao: "Abrir no Tenancy",
      acao: () => void ponte.abrirUrl(b.primeiraTarefa!.url),
    });
  const antes = estado.antesDoPasso1 ? "Antes dos três passos" : "Antes dos outros dois passos";
  return (
    <div className="boas">
      <div className="boas-lado">
        <Catavento tamanho={72} cor="#F5F5F7" />
        <h1>Boas-vindas à Novasce, {b.firstName}.</h1>
        <p>{antes}, {NUMEROS[cartoes.length]} que ajudam no primeiro dia.</p>
      </div>
      <section className="boas-corpo">
        {cartoes.map((c, n) => (
          <div key={c.titulo} className="cartao cartao-linha">
            <span className="num">{n + 1}</span>
            <div style={{ flex: 1 }}><b>{c.titulo}</b><br />{c.texto}</div>
            <Botao variante={c.primario ? "primario" : "secundario"} onClick={c.acao}>{c.botao}</Botao>
          </div>
        ))}
        <div className="acoes">
          <Botao variante="primario" onClick={async () => { await ponte.boasVindasVista(); aoTerminar(); }}>Vamos para os passos</Botao>
          <span style={{ fontSize: 13, color: "var(--muted)" }}>Dá para voltar aqui pelo dock.</span>
        </div>
      </section>
    </div>
  );
}
