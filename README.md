# Test Craft

Crie questões calculadas, organize seu banco e gere provas e gabaritos individualizados em PDF. Gratuito, em português e com os dados salvos no computador. Feito com o Art Craft.

## Instalar

Baixe **test-craft-0.3.0-windows-x64.msi** em [Releases](https://github.com/jorgewattes/test-craft/releases/latest), execute o instalador e abra **Test Craft** pelo menu Iniciar. Requer Windows 10/11 de 64 bits e suporte a OpenGL.

Para atualizar, feche o aplicativo e execute o novo MSI. Os dados ficam em **%LOCALAPPDATA%\Test Craft\data** e são preservados na atualização e desinstalação. Copie essa pasta para fazer backup.

A versão portátil em ZIP também está disponível: extraia e abra **test-craft.exe**. Nela, os dados ficam na pasta **data** ao lado do executável. Para migrar do ZIP ao MSI, feche o aplicativo e copie essa pasta para o caminho acima.

## Usar

- **Banco de questões:** importe XML Moodle ou JSON, pesquise, edite e exporte seu banco.
- **Elaborar questão:** escreva enunciado, variáveis e alternativas. Use `{x}` para variáveis e `{={x}*2}` para cálculos.
- **Criar prova:** escolha categorias, informe os alunos, confira a prévia e exporte PDFs e gabaritos.

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

O projeto fica em **C:\dev\moodtest2\app**, branch **main**, versão **0.3.0** e tag **v0.3.0**, no repositório **jorgewattes/test-craft**. Confira a conexão antes de enviar; a publicação só está concluída quando o workflow da tag terminar e a Release com MSI e ZIP estiver disponível no GitHub. A validação ICE do MSI é obrigatória no workflow; pacotes locais gerados com `-SkipMsiValidation` não substituem essa validação.

Com Rust e Visual Studio Build Tools instalados:

```powershell
cargo test --locked
cargo build --locked
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-release.ps1
```

O script gera MSI, ZIP, SHA-256 e as notas da versão em **dist/**. Tags `vX.Y.Z` correspondentes à versão do Cargo.toml publicam os pacotes pelo GitHub Actions, com as notas da versão atual. O workflow exige a validação do instalador antes da publicação.

Licença [MIT](LICENSE). Dependências e créditos em [CREDITS.md](CREDITS.md).
