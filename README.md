# Test Craft

Configure categorias pelo botão de ícone ao lado do filtro. A janela permite criar, renomear, reorganizar e excluir categorias, transferindo suas questões para outro destino.

Selecione vários arquivos JSON/XML de uma vez. Na conferência, altere o destino de cada categoria ou ajuste a categoria de uma questão individualmente antes de confirmar. Confira as quantidades, os erros e as categorias antes de confirmar. Questões com erro podem ser incluídas para revisão no editor ou deixadas de fora. Os caminhos das categorias são preservados ao exportar e importar.

Em **Criar prova**, adicione um logo PNG ou JPEG se desejar. O logo é opcional, fica salvo nas preferências e acompanha o registro do lote; provas antigas continuam abrindo sem imagem.

No editor, escolha **Rascunho** ou **Pronta**. Somente questões prontas e válidas são contabilizadas e sorteadas nas provas. Em **Questões por categoria**, escolha **Sorteio por aluno** para selecionar questões independentemente para cada aluno ou **Sorteio por prova** para sortear uma seleção comum, com valores calculados por aluno. Os valores são sorteados de forma independente e podem coincidir.

Use **Adicionar categoria ou subcategoria** e a busca para montar a lista da prova; cada categoria inclui apenas as questões diretamente nela. O botão **×** retira uma categoria da prova. As setas da categoria mudam a ordem dos blocos. Depois de **Conferir prévia**, use as setas de cada questão para adiantar ou postergar sua posição em todas as versões. Essa ordem acompanha os PDFs, os gabaritos e o registro do lote e é reaplicada ao gerar outra prévia com o mesmo plano.

A importação Moodle mantém imagens PNG/JPEG do enunciado e das alternativas, incluindo arquivos incorporados em subpastas, nomes codificados em URLs, imagens em base64 no HTML e o formato antigo `image_base64`. As imagens aparecem nas prévias e nos PDFs. Cada referência sem arquivo incorporado gera um aviso para revisão.

Na configuração de categorias, marque várias caixas para mover as categorias e suas subcategorias para dentro de outra categoria ou apagar todas as selecionadas com suas questões após confirmação.

Crie questões calculadas, organize seu banco e gere provas e gabaritos individualizados em PDF. Gratuito, em português e com os dados salvos no computador. Feito com o Art Craft.

## Instalar

A versão **0.4.3** é distribuída em MSI e ZIP **sem assinatura digital**, mantendo a distribuição da versão anterior. O Windows pode mostrar **Autor desconhecido**. Os arquivos SHA-256 permitem conferir a integridade dos downloads. Veja os detalhes em [SIGNING.md](SIGNING.md).

Baixe o [instalador MSI do Test Craft 0.4.3](https://github.com/jorgewattes/test-craft/releases/download/v0.4.3/test-craft-0.4.3-windows-x64.msi), execute o instalador e abra **Test Craft** pelo menu Iniciar. Requer Windows 10/11 de 64 bits e suporte a OpenGL.

A partir da versão 0.4.2, abra **Sobre e atualizações**, verifique a versão e clique em **Baixar e instalar**. O aplicativo baixa e confere o SHA-256, solicita autorização administrativa pelo Windows e fecha para instalar; ao concluir, abra-o novamente. Os dados ficam em **%LOCALAPPDATA%\Test Craft\data** e são preservados na atualização e desinstalação. Feche o aplicativo antes de copiar essa pasta para fazer backup.

A versão portátil em ZIP também está disponível: extraia e abra **test-craft.exe**. Nela, os dados ficam na pasta **data** ao lado do executável. Para migrar do ZIP ao MSI, feche o aplicativo e copie essa pasta para o caminho acima.

## Usar

- **Banco de questões:** importe XML Moodle ou JSON, pesquise, edite e exporte seu banco.
- **Elaborar questão:** escreva enunciado, variáveis e alternativas. Use `{x}` para variáveis e `{={x}*2}` para cálculos.
- **Criar prova:** escolha categorias, informe os alunos, confira a prévia e exporte PDFs e gabaritos.

Ao criar uma categoria no banco, a lista continua mostrando todas as categorias. Para renomear uma categoria ou alterar sua categoria principal, abra o ícone de configuração ao lado do dropdown e selecione-a na janela; as questões e subcategorias acompanham a alteração. A exclusão de categorias ocupadas exige um destino para as questões.

Marque as caixas das questões ou use **Selecionar resultados** para agir em lote: exportar a seleção em JSON, copiar questões para uma categoria, mover ou apagar. A seleção abrange todos os resultados da busca, incluindo outras páginas. **Copiar JSON** envia a seleção à área de transferência. O contador considera também seleções em outros filtros; **Limpar seleção** desmarca tudo. A exclusão pede confirmação e mostra as questões afetadas.

O banco local usa **banco.sqlite3**, com índices por categoria e gravações individuais em transações. Na primeira abertura, o **banco.json** existente é migrado automaticamente e preservado como cópia anterior à migração; as alterações seguintes ficam no SQLite. JSON continua disponível para importar e exportar. A lista mostra 50 questões por página, mantém a busca em cache e valida questões novas ou alteradas em segundo plano. Se outra janela alterar o banco, reabra o aplicativo antes de salvar para evitar sobrescrever dados.

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

O projeto fica em **C:\dev\moodtest2\app**, branch **main**, versão **0.4.3** e tag planejada **v0.4.3**, no repositório **jorgewattes/test-craft**. Confira a conexão antes de enviar; a publicação só está concluída quando o workflow da tag terminar e a Release com MSI e ZIP estiver disponível no GitHub. A validação ICE do MSI é obrigatória no workflow; pacotes locais gerados com `-SkipMsiValidation` não substituem essa validação.

Com Rust e Visual Studio Build Tools instalados:

```powershell
cargo test --locked
cargo build --locked
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-release.ps1
```

O script gera MSI, ZIP, SHA-256, relatório de assinatura e as notas da versão em **dist/**. A configuração `unsignedReleaseVersion: "0.4.3"` em **release-config.json** autoriza somente essa versão sem certificado, inclusive no GitHub Actions. Outras versões exigem assinatura por padrão; a preparação local sem certificado precisa de `-AllowUnsigned`. Configure a assinatura conforme [SIGNING.md](SIGNING.md). Tags `vX.Y.Z` correspondentes à versão do Cargo.toml publicam os pacotes com as notas da versão atual e validação ICE obrigatória do instalador.

Licença [MIT](LICENSE). Dependências e créditos em [CREDITS.md](CREDITS.md).
