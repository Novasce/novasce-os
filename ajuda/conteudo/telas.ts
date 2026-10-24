import type { AjudaDeTela } from "../tipos";

export const TELAS: AjudaDeTela[] = [
  {
    tela: "comecar", nome: "Começar",
    titulo: "O Começar liga a sua conta do Tenancy, a sua IA e o seu brain.",
    resumo: "Leva uns 10 minutos. Dá para pular um passo e voltar depois: ele reabre onde parou.",
    perguntas: [
      { titulo: "O que cada passo faz", resposta: "O passo 1 liga o Tenancy. O passo 2 instala a IA que você já assina e abre o login dela. O passo 3 conecta o GitHub, cria o seu brain e traz os espaços de que você participa." },
      { titulo: "Não tenho conta no GitHub", resposta: "No passo 3, clique em Não tenho conta: criar no GitHub, de graça. Crie a conta com o seu e-mail da Novasce e volte ao Começar." },
      { titulo: "Ainda não tenho IA", resposta: "Pule o passo 2. Tudo continua funcionando. Quando você assinar uma IA, abra o Começar por aqui e faça o passo 2." },
    ],
  },
  {
    tela: "geral", nome: "Área de trabalho",
    titulo: "A barra do alto tem quatro sinais: falar, nuvem, senhas e ajuda.",
    resumo: "Passe o mouse em cada um para ver o nome. A busca do centro acha app, nota do brain e arquivo.",
    perguntas: [
      { titulo: "A nuvem ficou laranja", resposta: "Algo não subiu. Clique na nuvem para ver o quê. Nada se perde: o arquivo continua neste computador." },
      { titulo: "Como acho uma nota", resposta: "Abra a busca e escreva uma palavra da nota. Ela abre no Obsidian." },
      { titulo: "Como instalo um app", resposta: "Abra Recomendados da casa, aqui na Ajuda. Instalar não pede senha de administrador." },
    ],
  },
  {
    tela: "arquivos", nome: "Arquivos",
    titulo: "Documentos fica salvo no seu Google Drive. O brain fica salvo no GitHub.",
    resumo: "O sinal de nuvem da barra diz se está salvo, subindo ou com problema.",
    perguntas: [
      { titulo: "Arquivo de RH vai onde?", resposta: "No Tenancy, no cadastro do colaborador. Nunca no Drive." },
      { titulo: "Apareceu uma cópia com o nome do computador", resposta: "O mesmo arquivo mudou em dois lugares. Nada foi apagado: abra as duas cópias e fique com a certa." },
    ],
  },
  {
    tela: "brain", nome: "Brain",
    titulo: "O brain é a sua pasta de notas, guardada no seu GitHub, privada.",
    resumo: "Ele salva sozinho a cada poucos minutos. Antes de cada envio, uma varredura procura senhas e segura o arquivo que tiver uma.",
    perguntas: [
      { titulo: "Um arquivo ficou só neste computador", resposta: "Ele parece ter uma senha. Tire a senha do arquivo e ele sobe sozinho na próxima vez." },
      { titulo: "O que é um espaço", resposta: "O brain de uma área da Novasce, na pasta Espaços. Um espaço nunca lê o seu brain." },
    ],
  },
  {
    tela: "navegador", nome: "Navegador",
    titulo: "O Chrome já vem com o Tenancy, as IAs e a extensão do Bitwarden.",
    resumo: "Para as senhas da casa, use o Cofre do Tenancy. Para as suas, o Bitwarden.",
    perguntas: [{ titulo: "Esqueci a senha de um site", resposta: "Clique no sinal de senhas da barra e procure no Bitwarden." }],
  },
  {
    tela: "loja", nome: "Loja",
    titulo: "Instalar um app não pede senha de administrador.",
    resumo: "O app entra só para você. Os Recomendados da casa também estão aqui na Ajuda.",
    perguntas: [{ titulo: "Não achei o app", resposta: "Peça ajuda dizendo o nome do app." }],
  },
  {
    tela: "sala", nome: "Sala",
    titulo: "A Sala é a tela da TV da sede. Ela nasce discreta.",
    resumo: "Para ver tudo, entre com o seu login pelo celular. A Sala mostra só o que o seu papel vê, por 10 minutos.",
    perguntas: [
      { titulo: "A Sala voltou sozinha", resposta: "Com 10 minutos sem uso, a conta Sede volta à Sala e o navegador esquece tudo." },
      { titulo: "Minha conta trancou", resposta: "Na sede, a sua conta tranca com 5 minutos sem uso e fica por trás, com tudo aberto. Entre de novo com a sua senha." },
    ],
  },
];
