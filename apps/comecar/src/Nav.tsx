import { AnelProgresso } from "@novasce/ui";
import type { EstadoTela, Tela } from "./tipos";

export function resumoDosPassos(feitos: number, tela: Tela): [string, string] {
  if (tela === "espacos") return ["Quase 3 de 3", "faltam os espaços"];
  const falta = ["leva uns 10 minutos", "faltam a IA e o GitHub", "falta GitHub e brain", "tudo feito"][feitos] ?? "";
  return [`${feitos} de 3 passos`, falta];
}

const PASSOS: { nome: string; telas: Tela[] }[] = [
  { nome: "Tenancy", telas: ["passo1"] },
  { nome: "IA", telas: ["passo2"] },
  { nome: "GitHub e brain", telas: ["passo3", "espacos"] },
];

export function Nav({ estado }: { estado: EstadoTela }) {
  const [titulo, sub] = resumoDosPassos(estado.passosFeitos, estado.tela);
  return (
    <nav className="nav-c" aria-label="Passos do Começar">
      <div className="quem">{estado.pessoa ? <><b>{estado.pessoa.firstName}</b><span>{estado.pessoa.email}</span></> : null}</div>
      <div className="resumo-passos">
        <AnelProgresso passos={estado.passosFeitos as 0 | 1 | 2 | 3} tamanho={64} />
        <div><b>{titulo}</b><span>{sub}</span></div>
      </div>
      <div className="passos">
        {PASSOS.map((p, n) => {
          const feito = n < estado.passosFeitos && !p.telas.includes(estado.tela);
          const agora = p.telas.includes(estado.tela);
          return (
            <span key={p.nome} className={feito ? "feito" : agora ? "agora" : "falta"}>
              <i aria-hidden="true">{feito ? "✓" : agora ? n + 1 : ""}</i>
              {p.nome}
            </span>
          );
        })}
      </div>
      <div className="rodape-nav">Dá para pular e voltar. O Começar fica no dock.</div>
    </nav>
  );
}
