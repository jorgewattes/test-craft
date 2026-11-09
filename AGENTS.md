# GitHub e verificacao

Este projeto usa `https://github.com/jorgewattes/test-craft.git` como `origin`.
A configuracao da publicacao fica em `release-config.json`.

Antes de enviar atualizacoes, execute `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/setup-github.ps1 -CheckOnly`.
No Craft Anything, o perfil `Codex Caderno GitHub` usa a ferramenta MCP do Craft `build_command_run` para as operacoes de GitHub. A ferramenta executa na pasta do aplicativo com a conta Windows que abriu o Craft. Use-a para o `setup-github.ps1 -CheckOnly` e para os comandos Git de publicacao solicitados pelo usuario. Se o Git apontar proprietario diferente, use `git -c safe.directory=C:/dev/moodtest2/app` somente nesse comando. Para publicar, mostre a branch, o commit e a tag e aguarde o cartao de aprovacao do Craft.
Se a verificacao foi executada em `CodexSandboxOffline`, repita-a pelo executor do Craft. Se essa ferramenta estiver indisponivel, encaminhe o envio para uma sessao do Codex ou do PowerShell aberta pelo usuario do Windows conectado ao GitHub.
Se a autenticacao falhar tambem nessa sessao, oriente a pessoa a abrir `configurar-github.cmd` e concluir a autorizacao no navegador oficial do GitHub. Nunca solicite senhas ou tokens no chat.
Para instalar a configuracao local do Git e verificar a conta ja conectada, execute `scripts/setup-github.ps1` sem argumentos. Ele usa o GitHub CLI como provedor de credenciais somente neste repositorio e preenche a identidade local ausente com a conta autenticada.

O setup apenas configura e verifica acesso; publicar exige que a pessoa solicite o envio. Preserve alteracoes existentes e os bancos locais. Nao inclua `data/`, `.tools/`, `.env`, credenciais, `target/` ou `dist/` nos commits; MSI e ZIP sao anexos de Releases.

Valide mudancas com `cargo test --locked` e `cargo build --release --locked`.
Para preparar MSI e ZIP, use `scripts/build-release.ps1`; a publicacao por tag `vX.Y.Z` usa `.github/workflows/release.yml` e exige a validacao ICE do instalador. O parametro `-SkipMsiValidation` serve apenas para preparacao local em ambientes sem Windows Installer; nao equivale a validar a instalacao.
Execute `rebuild_check` pelo Craft Anything depois de mudancas no aplicativo e informe qualquer erro ou indisponibilidade sem declarar aprovacao. Em um pedido apenas de publicacao, use os testes e o empacotamento do release; dispense uma nova chamada manual de `rebuild_check`.
