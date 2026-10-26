import type { Guia } from "../tipos";

export const GUIAS: Guia[] = [
  {
    id: "primeiro-dia", titulo: "Primeiro dia", resumo: "Em três passos, a máquina fica sua.",
    passos: [
      { titulo: "1. Entrar no Tenancy", oQueFazer: "Use o Começar, que já está aberto na tela.", ondeClicar: "Em Entrar no Tenancy, e depois no seu e-mail da Novasce.", oQueDeveAparecer: "O seu nome no alto e o primeiro terço do contorno fechado.", exemplo: "Tela do Começar" },
      { titulo: "2. Escolher a sua IA", oQueFazer: "Marque a IA que você já assina.", ondeClicar: "Em Instalar e entrar.", oQueDeveAparecer: "O login da IA aberto e o segundo terço do contorno fechado.", exemplo: "Passo 2 do Começar" },
      { titulo: "3. Conectar o GitHub e criar o brain", oQueFazer: "Confira o código e aprove no navegador que abriu.", ondeClicar: "Em Authorize no GitHub, e depois em Criar meu brain.", oQueDeveAparecer: "Meu brain pronto e a lista dos seus espaços.", exemplo: "Passo 3 do Começar" },
      { titulo: "4. Trazer os espaços", oQueFazer: "Deixe marcados os espaços de que você participa.", ondeClicar: "Em Trazer o Studio.", oQueDeveAparecer: "O contorno inteiro fechado e a frase Pronto. Os três passos estão feitos." },
    ],
  },
  {
    id: "pessoa-nova", titulo: "Pessoa nova na Novasce", resumo: "No primeiro dia, o Começar mostra a casa, a sua área e a primeira tarefa.",
    passos: [
      { titulo: "A casa", oQueFazer: "Leia o Manual da Novasce primeiro.", ondeClicar: "Em Ler agora.", oQueDeveAparecer: "O Manual aberto no Material do time." },
      { titulo: "Sua área", oQueFazer: "Conheça o documento da sua área.", ondeClicar: "Em Abrir.", oQueDeveAparecer: "O documento da área, com quem cuida dela." },
      { titulo: "Sua primeira tarefa", oQueFazer: "Abra a tarefa que já está no seu quadro.", ondeClicar: "Em Abrir no Tenancy.", oQueDeveAparecer: "A tarefa aberta no Tenancy, com o prazo." },
      { titulo: "Os passos", oQueFazer: "Siga para os passos do Começar.", ondeClicar: "Em Vamos para os passos.", oQueDeveAparecer: "O passo que falta, à esquerda." },
    ],
  },
  {
    id: "a-rotina", titulo: "A rotina", resumo: "O dia abre no Meu dia e o resto salva sozinho.",
    passos: [
      { titulo: "Chegar de manhã", oQueFazer: "Entre na sua conta.", ondeClicar: "Em nada: o Meu dia abre sozinho na primeira entrada do dia.", oQueDeveAparecer: "Bom dia, com a coisa que fecha hoje." },
      { titulo: "Saber se salvou", oQueFazer: "Olhe a nuvem na barra do alto.", ondeClicar: "Em nada, ou na nuvem para ver os detalhes.", oQueDeveAparecer: "A nuvem verde e a frase salvos há poucos minutos." },
      { titulo: "Falar em vez de digitar", oQueFazer: "Ponha o cursor no campo de texto.", ondeClicar: "Segure Super + Espaço e fale. Solte para escrever.", oQueDeveAparecer: "Ouvindo · solte para escrever, e depois o texto no campo." },
    ],
  },
  {
    id: "brain-e-espacos", titulo: "Brain e espaços", resumo: "O brain é seu. Os espaços são das áreas.",
    passos: [
      { titulo: "Abrir o brain", oQueFazer: "Abra o Obsidian.", ondeClicar: "No Obsidian, no dock.", oQueDeveAparecer: "As suas notas da pasta Brain." },
      { titulo: "Abrir um espaço", oQueFazer: "Abra a pasta do espaço.", ondeClicar: "Em Arquivos, na pasta Espaços.", oQueDeveAparecer: "Uma pasta por espaço que você trouxe." },
      { titulo: "Levar algo para um espaço", oQueFazer: "Monte um pacote revisado com o que vai.", ondeClicar: "Em nada automático: um espaço nunca lê o seu brain.", oQueDeveAparecer: "O pacote no espaço, revisado por quem cuida dele." },
    ],
  },
  {
    id: "com-o-tenancy", titulo: "Com o Tenancy", resumo: "O Tenancy segura a casa. O computador só abre e conversa.",
    passos: [
      { titulo: "Abrir o Tenancy", oQueFazer: "Abra o Tenancy em janela própria.", ondeClicar: "No T, no dock.", oQueDeveAparecer: "O seu quadro, com o seu nome no alto." },
      { titulo: "Ligar a IA ao Tenancy", oQueFazer: "Crie a sua chave de IA no Tenancy e cole no Começar.", ondeClicar: "No passo 2 do Começar, em Ligar ao Tenancy.", oQueDeveAparecer: "Ligada, ao lado da IA." },
    ],
  },
  {
    id: "na-sede", titulo: "Na sede: Sala e conta Sede", resumo: "Qualquer um usa a conta Sede, nada fica, e a Sala sempre volta.",
    passos: [
      { titulo: "Usar a conta Sede", oQueFazer: "Saia da tela cheia da Sala.", ondeClicar: "No navegador que esquece, no dock.", oQueDeveAparecer: "Uma janela do Chrome sem nenhum login." },
      { titulo: "Voltar à Sala", oQueFazer: "Nada: com 10 minutos sem uso, ela volta sozinha.", ondeClicar: "Em Continuar usando, se você ainda estiver ali.", oQueDeveAparecer: "A Sala em tela cheia, e o navegador sem nada do que estava aberto." },
      { titulo: "Entrar na sua conta na sede", oQueFazer: "Troque de usuário.", ondeClicar: "Em Trocar usuário, e depois no seu nome.", oQueDeveAparecer: "A sua área de trabalho. Com 5 minutos sem uso, ela tranca e a TV volta à Sala." },
      { titulo: "Reiniciar para atualizar", oQueFazer: "Reinicie quando ninguém estiver usando.", ondeClicar: "Em Reiniciar, no canto da Sala.", oQueDeveAparecer: "O botão só fica ativo com os agentes fora de uma rodada." },
    ],
  },
  {
    id: "pedir-ajuda", titulo: "Pedir ajuda", resumo: "Um botão, e a Novasce recebe o chamado já com o print.",
    passos: [
      { titulo: "Abrir", oQueFazer: "Abra a ajuda na tela onde travou.", ondeClicar: "No ponto de interrogação da barra do alto.", oQueDeveAparecer: "A Central, com a ajuda da tela aberta." },
      { titulo: "Contar o que aconteceu", oQueFazer: "Escreva ou fale.", ondeClicar: "Em Pedir ajuda, e depois em Falar se quiser.", oQueDeveAparecer: "O print e os dados da máquina em Vai junto." },
      { titulo: "Enviar", oQueFazer: "Confira o que vai junto.", ondeClicar: "Em Enviar para a Novasce.", oQueDeveAparecer: "Chamado aberto. A resposta chega aqui e no Tenancy." },
      { titulo: "Responder", oQueFazer: "Quando a resposta chegar, diga se resolveu.", ondeClicar: "Em Resolveu ou Ainda não, no aviso.", oQueDeveAparecer: "O aviso some e a Novasce fica sabendo." },
    ],
  },
  {
    id: "senhas-e-seguranca", titulo: "Senhas e segurança", rotuloNoGuia: "Segurança", resumo: "Uma senha para lembrar: a do computador. O resto fica guardado.",
    passos: [
      { titulo: "As suas senhas", oQueFazer: "Guarde no Bitwarden.", ondeClicar: "No sinal de senhas da barra, em Abrir o Bitwarden.", oQueDeveAparecer: "O seu cofre, depois da senha mestra." },
      { titulo: "As senhas do time", oQueFazer: "Use o Cofre do Tenancy.", ondeClicar: "No sinal de senhas da barra, em Abrir o Cofre do time.", oQueDeveAparecer: "O Cofre no Tenancy. Cada senha revelada fica registrada." },
      { titulo: "Esqueceu a senha do computador", oQueFazer: "Use o código da tela de login.", ondeClicar: "Aponte o celular para o código do canto da tela de login.", oQueDeveAparecer: "Um chamado no Tenancy já dizendo qual é a máquina. Só o Eduardo troca a senha." },
    ],
  },
  {
    id: "sem-internet", titulo: "Sem internet", resumo: "O que continua funcionando e o que espera a conexão.",
    passos: [
      { titulo: "Continua funcionando", oQueFazer: "Siga trabalhando.", ondeClicar: "No brain, nos Documentos já baixados, na voz e nos apps instalados.", oQueDeveAparecer: "Tudo abre normalmente." },
      { titulo: "Espera a conexão", oQueFazer: "Nada: o que você pedir fica guardado.", ondeClicar: "Em nada. Para pedir ajuda, use Pedir ajuda mesmo assim.", oQueDeveAparecer: "O pedido ficou guardado neste computador e sai sozinho quando a conexão voltar." },
    ],
  },
  {
    id: "problemas-comuns", titulo: "Problemas comuns", resumo: "O que aparece e o que fazer.", passos: [],
    problemas: [
      { oQueAparece: "A nuvem da barra ficou laranja.", oQueFazer: "Clique nela para ver o que não subiu. O arquivo continua neste computador." },
      { oQueAparece: "Um arquivo do brain ficou só neste computador.", oQueFazer: "Ele parece ter uma senha. Tire a senha e ele sobe sozinho." },
      { oQueAparece: "Apareceu uma cópia com o nome do computador.", oQueFazer: "O arquivo mudou em dois lugares. Abra as duas e fique com a certa." },
      { oQueAparece: "O passo 1 precisa de internet.", oQueFazer: "Conecte o Wi-Fi. O Começar continua de onde parou." },
      { oQueAparece: "O ditado não escreveu no campo.", oQueFazer: "O texto ficou copiado. Cole com Ctrl+V." },
      { oQueAparece: "O chamado não foi enviado.", oQueFazer: "Nada: ele sai sozinho quando a conexão voltar." },
    ],
  },
  {
    id: "por-onde-comecar", titulo: "Por onde começar", resumo: "Três caminhos, do mais rápido ao mais completo.",
    passos: [
      { titulo: "Fazer o Começar", oQueFazer: "Siga os três passos.", ondeClicar: "No Começar, no dock.", oQueDeveAparecer: "Pronto. Os três passos estão feitos." },
      { titulo: "Ler este guia", oQueFazer: "Leia Primeiro dia e A rotina.", ondeClicar: "Na lista Neste guia, à esquerda.", oQueDeveAparecer: "Os passos com o que fazer, onde clicar e o que deve aparecer." },
      { titulo: "Pedir ajuda", oQueFazer: "Se algo não bateu, peça ajuda dizendo a seção.", ondeClicar: "No ponto de interrogação da barra do alto.", oQueDeveAparecer: "Chamado aberto." },
    ],
  },
];

export const GUIAS_DA_CENTRAL = ["primeiro-dia", "pessoa-nova", "brain-e-espacos", "na-sede", "senhas-e-seguranca", "sem-internet", "problemas-comuns"];
export const SECOES_DO_GUIA = ["primeiro-dia", "a-rotina", "com-o-tenancy", "pedir-ajuda", "senhas-e-seguranca", "problemas-comuns", "por-onde-comecar"];
