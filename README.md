A exportação da prova salva somente **todas_as_provas.pdf** e **gabarito_consolidado.pdf**. O histórico com valores, alternativas e logo fica na pasta de dados interna do aplicativo; registros antigos continuam abrindo. A prévia compartilha a paginação, as margens, as tabelas, as imagens e as posições do gerador PDF. As fontes podem variar ligeiramente na tela. Rascunhos têm botão e borda em amarelo claro.

Em **Criar prova**, cole os alunos na grade: textos com tabulações viram colunas e quebras de linha viram linhas. Marque a coluna dos nomes, remova colunas/linhas extras pelo **×** e marque **Primeira linha é cabeçalho** quando necessário. Só a coluna escolhida entra na prova; a grade completa fica salva nas preferências e listas antigas são migradas automaticamente.

# Test Craft

Em **Criar prova**, o seletor de categorias mostra automaticamente os níveis com questões prontas e válidas: **Exemplo**, **Fácil**, **Média**, **Difícil**, **Desafio** e **Sem classificação**. Escolher um nível inclui apenas questões dessa classificação, inclusive nas subcategorias. Os níveis são opções da prova e não alteram as categorias do banco. Selecione **Todas desta categoria** para incluir os demais níveis; os já escolhidos não são duplicados.

Configure categorias pelo botão de ícone ao lado do filtro. A janela permite criar, renomear, reorganizar e excluir categorias, transferindo suas questões para outro destino.

O banco mostra os totais em uma linha compacta. Combine categoria e busca com os filtros **Todas**, **Prontas**, **Com erro** e **Rascunhos**. Questões com validação pendente não aparecem como prontas nem com erro até a conferência terminar; rascunhos com problemas também aparecem em **Com erro**. Em **Criar prova**, o botão **Adicionar categoria ou subcategoria** abre uma janela de busca que aceita palavras sem acentos e mantém o foco durante a digitação.

**Exportar** e **Exportar seleção** permitem salvar JSON completo ou **Moodle XML**. O XML exporta apenas questões prontas e válidas, preservando categorias, imagens PNG/JPEG e gabaritos, com suporte a uma ou várias respostas corretas. Cada questão calculada vira uma questão de múltipla escolha com valores fixos; use JSON para preservar as fórmulas editáveis. Em Moodle, importe pelo banco de questões usando o formato **Moodle XML** e a opção de categorias do arquivo.

Selecione vários arquivos JSON/XML de uma vez. Na conferência, altere o destino de cada categoria ou ajuste a categoria de uma questão individualmente antes de confirmar. Confira as quantidades, os erros e as categorias antes de confirmar. Questões com erro podem ser incluídas para revisão no editor ou deixadas de fora. Os caminhos das categorias são preservados ao exportar e importar.

Em **Criar prova**, adicione um logo PNG ou JPEG se desejar. O logo é opcional, fica salvo nas preferências e acompanha o registro do lote; provas antigas continuam abrindo sem imagem.

No editor, escolha **Rascunho** ou **Pronta**. Somente questões prontas e válidas são contabilizadas e sorteadas nas provas. Em **Questões por categoria**, escolha **Sorteio por aluno** para selecionar questões independentemente para cada aluno ou **Sorteio por prova** para sortear uma seleção comum, com valores calculados por aluno. Os valores são sorteados de forma independente e podem coincidir.

Use **Adicionar categoria ou subcategoria** e a busca para montar a lista da prova; selecionar uma categoria principal adiciona as subcategorias disponíveis em linhas separadas, sem duplicar as já escolhidas. As quantidades de sorteio continuam individuais por linha, somente para questões diretamente nela. O seletor agrupa as subcategorias em seções recolhíveis; os números mostram as questões prontas disponíveis. A coluna Disponíveis mostra apenas as prontas e válidas diretamente na linha. O botão **×** retira uma categoria da prova. As setas da categoria mudam a ordem dos blocos. **Conferir prévia** abre uma janela popup com páginas na proporção do PDF, zoom e seleção de aluno. Em **Reordenar questões em todas as versões**, use as setas para ajustar as posições. Essa ordem acompanha os PDFs, os gabaritos e o registro do lote e é reaplicada ao gerar outra prévia com o mesmo plano.

A importação Moodle mantém imagens PNG/JPEG do enunciado e das alternativas, incluindo arquivos incorporados em subpastas, nomes codificados em URLs, imagens em base64 no HTML e o formato antigo `image_base64`. As imagens aparecem nas prévias e nos PDFs. Cada referência sem arquivo incorporado gera um aviso para revisão.

Na configuração de categorias, marque várias caixas para mover as categorias e suas subcategorias para dentro de outra categoria ou apagar todas as selecionadas com suas questões após confirmação.

Crie questões calculadas, organize seu banco e gere provas e gabaritos individualizados em PDF. Gratuito, em português e com os dados salvos no computador. Feito com o Art Craft.

## Instalar

A versão **0.4.6** é distribuída em MSI e ZIP **sem assinatura digital**, mantendo a distribuição da versão anterior. O Windows pode mostrar **Autor desconhecido**. Os arquivos SHA-256 permitem conferir a integridade dos downloads. Veja os detalhes em [SIGNING.md](SIGNING.md).

Baixe o [instalador MSI do Test Craft 0.4.6](https://github.com/jorgewattes/test-craft/releases/download/v0.4.6/test-craft-0.4.6-windows-x64.msi), execute o instalador e abra **Test Craft** pelo menu Iniciar. Requer Windows 10/11 de 64 bits e suporte a OpenGL.

A partir da versão 0.4.6, o programa verifica releases ao iniciar, recupera configurações antigas com repositório vazio e tenta novamente após falhas de conexão. Uma versão nova aparece na janela **Nova versão disponível**; clique em **Baixar e instalar** para concluir. A instalação precisa dessa confirmação e da autorização do Windows quando solicitada. A verificação pode ser desativada em **Sobre e atualizações**.

A partir da versão 0.4.2, abra **Sobre e atualizações**, verifique a versão e clique em **Baixar e instalar**. O aplicativo baixa e confere o SHA-256, solicita autorização administrativa pelo Windows e fecha para instalar; ao concluir, abra-o novamente. Os dados ficam em **%LOCALAPPDATA%\Test Craft\data** e são preservados na atualização e desinstalação. Feche o aplicativo antes de copiar essa pasta para fazer backup.

A versão portátil em ZIP também está disponível: extraia e abra **test-craft.exe**. Nela, os dados ficam na pasta **data** ao lado do executável. Para migrar do ZIP ao MSI, feche o aplicativo e copie essa pasta para o caminho acima.

## Usar

As figuras dos enunciados ficam centralizadas na prévia, no PDF e no Moodle XML.
Use `{={x}*rnd(0.8,1.2)}` para multiplicar por um fator aleatório entre 0,8 e 1,2. Em `rnd(min,max)`, a vírgula separa os dois limites; use ponto nos limites decimais. Mínimo e máximo são inclusivos, podem ser iguais ou negativos e devem estar em ordem. Chamadas com os mesmos limites numéricos reutilizam o fator dentro da mesma questão; cada versão sorteia novamente. O fator aparece nos valores da prévia e fica registrado no lote.

Quando alternativas coincidem, o aviso mostra suas letras, os textos ou fórmulas e os valores exatos do teste. A prévia no banco e no editor usa a combinação que falhou, com as alternativas repetidas destacadas, inclusive na prévia ampliada. Alterar a questão limpa esse exemplo; valide novamente para conferir todos os intervalos. Questões com `rnd` são conferidas em 256 amostras e cada versão gerada é validada antes de salvar.


- **Banco de questões:** importe XML Moodle ou JSON, pesquise, edite e exporte seu banco.
- **Elaborar questão:** escreva enunciado, variáveis e alternativas. Use `{x}` para variáveis e `{={x}*2}` para cálculos.
- **Criar prova:** escolha categorias, informe os alunos, confira a prévia e exporte PDFs e gabaritos.

Ao criar uma categoria no banco, a lista continua mostrando todas as categorias. Para renomear uma categoria ou alterar sua categoria principal, abra o ícone de configuração ao lado do dropdown e selecione-a na janela; as questões e subcategorias acompanham a alteração. A exclusão de categorias ocupadas exige um destino para as questões.

Marque as caixas das questões ou use **Selecionar resultados** para agir em lote: exportar a seleção em JSON, copiar questões para uma categoria, mover ou apagar. A seleção abrange todos os resultados da busca, incluindo outras páginas. **Copiar JSON** envia a seleção à área de transferência. O contador considera também seleções em outros filtros; **Limpar seleção** desmarca tudo. A exclusão pede confirmação e mostra as questões afetadas.

O banco local usa **banco.sqlite3**, com índices por categoria e gravações individuais em transações. Na primeira abertura, o **banco.json** existente é migrado automaticamente e preservado como cópia anterior à migração; as alterações seguintes ficam no SQLite. JSON continua disponível para importar e exportar. A lista mostra 50 questões por página, mantém a busca em cache e valida questões novas ou alteradas em segundo plano. Gravações sem mudanças não geram uma nova revisão do banco. A lista do histórico usa resumos em memória e é recarregada pelo botão **Atualizar**; as provas completas e imagens são abertas ao pedir a prévia. Se outra janela alterar o banco, reabra o aplicativo antes de salvar para evitar sobrescrever dados.

Os PDFs usam margens de 1,5 cm, espaçamento compacto, cabeçalho com identificação do aluno e numeração no rodapé. Questões que cabem em uma página ficam juntas com suas alternativas.

Questões com erro são importadas e salvas com **borda vermelha** e indicação do problema. Corrija no editor; problemas específicos da importação podem ser marcados como revisados após a correção. O XML original pode ser copiado no editor. Questões precisam estar válidas para gerar provas. XML com estrutura ilegível é informado como erro de arquivo.

## Desenvolvimento

### Conectar o Codex ao GitHub

Abra **configurar-github.cmd** nesta pasta e conclua a autorização no navegador oficial do GitHub com uma conta que possa atualizar [jorgewattes/test-craft](https://github.com/jorgewattes/test-craft). Requer Git e GitHub CLI (`gh`) instalados. Depois, peça ao Codex para enviar as atualizações. O atalho verifica a permissão de escrita, configura o Git somente para este projeto e preenche a identidade de commits ausente com a conta conectada. Nenhuma atualização é publicada durante a configuração.

Para conferir a conexão sem abrir o login:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/setup-github.ps1 -CheckOnly
```

Se aparecer `detected dubious ownership`, abra novamente **configurar-github.cmd** atualizado. O atalho autoriza somente a pasta deste projeto durante os comandos de configuração, sem alterar a confiança global do Git.

Se o login expirar, abra novamente o atalho. Não envie senhas ou tokens pelo chat. As instruções para próximas sessões do Codex estão em **AGENTS.md**. Consulte também a [documentação oficial do Codex](https://developers.openai.com/codex/).

No Craft Anything, o perfil **Codex Caderno GitHub** usa o executor do Craft (`build_command_run`) com seu usuário habitual do Windows. Se o terminal isolado informar **CodexSandboxOffline**, o Codex deve repetir a verificação por esse executor antes de solicitar um novo login. Se o executor estiver indisponível, abra o Codex ou o PowerShell com seu usuário habitual do Windows.

O projeto fica em **C:\dev\moodtest2\app**, branch **main**, no repositório **jorgewattes/test-craft**. A versão **0.4.6** já foi publicada com MSI, ZIP e SHA-256; os dados da publicação estão em **release-verification.json**. As melhorias posteriores na branch **main** estão descritas em **Em desenvolvimento** no CHANGELOG e ainda não fazem parte do instalador 0.4.6. A publicação por tag gera os pacotes no GitHub Actions; só está concluída quando o workflow terminar e os quatro arquivos estiverem disponíveis na Release. A validação ICE do MSI é obrigatória no workflow.

Com Rust e Visual Studio Build Tools instalados:

```powershell
cargo test --locked
cargo build --locked
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-release.ps1
```

O script gera MSI, ZIP, SHA-256, relatório de assinatura e as notas da versão em **dist/**. A configuração `unsignedReleaseVersion: "0.4.7"` em **release-config.json** autoriza somente essa versão sem certificado, inclusive no GitHub Actions. Outras versões exigem assinatura por padrão; a preparação local sem certificado precisa de `-AllowUnsigned`. Configure a assinatura conforme [SIGNING.md](SIGNING.md). Tags `vX.Y.Z` correspondentes à versão do Cargo.toml publicam os pacotes com as notas da versão atual e validação ICE obrigatória do instalador.

Licença [MIT](LICENSE). Dependências e créditos em [CREDITS.md](CREDITS.md).
