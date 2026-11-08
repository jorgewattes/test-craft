# Caderno

Aplicativo de código aberto para criar questões calculadas, organizar um banco local e gerar provas e gabaritos individualizados em PDF. Interface em português, sem conta e sem mensalidade. Versão atual: **0.2.0**.

## Usar no Windows

1. Baixe **caderno-0.2.0-windows-x64.zip** na página de Releases do repositório público (quando publicado).
2. Extraia o ZIP para uma pasta em que você possa gravar, como Documentos/Caderno. Evite executar dentro do ZIP ou instalar em Arquivos de Programas.
3. Abra **caderno.exe**. O aplicativo cria a pasta **data** ao lado do executável no primeiro uso.
4. Em **Banco de questões**, explore os exemplos ou importe seu banco. Em **Elaborar questão**, cadastre e valide suas questões. Em **Criar prova**, escolha quantidades por categoria, informe os alunos e confira a prévia antes de exportar.

Requisitos: Windows 10/11 de 64 bits, suporte gráfico OpenGL e permissão para gravar na pasta do programa. A consulta de atualizações utiliza o curl.exe fornecido pelo Windows. O executável é portátil e não tem assinatura digital. Não é necessário instalar Rust para usá-lo.

O pacote compilado nesta entrega está em **dist/caderno-0.2.0-windows-x64.zip** e o executável em **dist/caderno-0.2.0-windows-x64/caderno.exe**. O arquivo ZIP.sha256 contém a soma para conferir o download com Get-FileHash -Algorithm SHA256.

## Funcionalidades

- Banco de questões com busca, categorias, edição, duplicação, exclusão, importação e exportação JSON.
- Variáveis em intervalos discretos ou valores de datasets Moodle. Use **{x}** para inserir uma variável e **{={x}*2}** para um cálculo.
- Validação de nomes, intervalos, fórmulas, resultados finitos e alternativas duplicadas. Verifica todas as combinações até 10.000; acima disso, usa 256 amostras e avisa.
- Imagens PNG/JPG opcionais em enunciados e alternativas.
- Importação de XML Moodle calculatedmulti e multichoice, com datasets explícitos e imagens incorporadas.
- Quantidades e ordem por categoria, lista de alunos, identificador de sorteio e prévia por aluno.
- PDFs individuais de prova e gabarito, PDF com todas as provas, gabarito consolidado PDF/CSV e registro JSON dos valores e alternativas utilizados.
- Reabertura dos lotes a partir do registro JSON e atalhos para os arquivos locais.

## Dados, backup e troca de computador

Na distribuição portátil, **data/banco.json** armazena as questões, **data/preferencias.json** guarda a montagem da prova, **data/atualizacoes.json** guarda as preferências de atualização e **data/provas/** recebe os lotes exportados na pasta padrão. Durante desenvolvimento via Cargo, usa-se a pasta data do projeto.

Para backup completo, feche o programa e copie a pasta **data**. Para compartilhar somente questões, use **Exportar banco** e importe o JSON na outra instalação. Nomes de alunos e provas exportadas podem conter informações pessoais: compartilhe apenas o que desejar. Os scripts e o .gitignore deixam os dados locais fora do repositório e dos pacotes públicos.

## Atualizações

Em **Sobre e atualizações**, informe **conta/repositorio**, o link do repositório ou seu link de releases no GitHub e clique **Salvar preferências**. Os pacotes gerados pelo workflow oficial já incorporam o próprio repositório, dispensando essa configuração pelos usuários.

Com a opção automática ativa, o programa consulta a release estável mais recente ao abrir e a cada seis horas enquanto estiver em uso. Também é possível clicar **Verificar agora**. A consulta ocorre em segundo plano, tem limite de tempo e não envia questões, nomes de alunos ou provas. Falta de internet, limite da API, repositório inexistente e ausência de release são informados na tela; o trabalho local continua disponível.

Uma versão numericamente maior com o pacote **caderno-VERSAO-windows-x64.zip** publicado abre a oferta de atualização. **Baixar atualização** abre a página oficial da release no navegador; **Mais tarde** permite continuar. Prereleases e rascunhos são ignorados. A integração segue a [API oficial de releases do GitHub](https://docs.github.com/en/rest/releases/releases#get-the-latest-release).

Para instalar a atualização: salve seu trabalho, baixe e extraia o novo ZIP, feche o Caderno e substitua o executável antigo pelo novo, **mantendo a pasta data existente**. Não há substituição silenciosa de arquivos nem instalação automática. A configuração pública em release-config.json é incorporada durante a compilação; alterar esse arquivo depois de compilar não muda o executável. A preferência salva na interface prevalece sobre a configuração incorporada.

## Compilar e verificar

Instale Rust estável para o alvo **x86_64-pc-windows-msvc** e as ferramentas C++ do Visual Studio Build Tools com Windows SDK. Abra um terminal nesta pasta:

~~~powershell
cargo build --locked
cargo test --locked
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-release.ps1
~~~

O script testa, compila com **cargo rustc --release --locked -- -C target-feature=+crt-static**, incorpora estaticamente o runtime C, monta o ZIP em dist e calcula seu SHA-256. Para já incorporar o repositório público:

~~~powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-release.ps1 -Repository sua-conta/caderno
~~~

Com as dependências já disponíveis no computador, também é possível usar cargo build --release --locked --offline. Cargo.lock está incluído para manter as versões de dependências.

Verificações adicionais:

~~~powershell
cargo run --locked -- --self-check
cargo run --locked -- --screenshot shots/banco.png
cargo run --locked -- --screenshot shots/atualizacoes.png --page 4
~~~

A opção self-check gera PDFs de exemplo em checks. A captura encerra o aplicativo após salvar o PNG. Os testes cobrem cálculos, validação, importação, persistência, sorteio, PDFs e detecção de atualizações (versões, resposta inválida, pacote ausente e URLs).

## Publicar o repositório e a release

**Estado desta entrega:** fontes e distribuição Windows preparados localmente; a publicação remota depende de uma conta autenticada no GitHub. Não há um endereço público fictício configurado. O release-config.json permanece sem repositório até a publicação ou configuração pela interface.

1. Instale Git e GitHub CLI; autentique sua conta com **gh auth login**.
2. Execute o comando abaixo nesta pasta, escolhendo um repositório novo ou vazio:

~~~powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/publish.ps1 -Repository sua-conta/caderno
~~~

O script cria um repositório **público**, configura origin, incorpora seu endereço no aplicativo, testa e compila, registra os fontes e envia a tag correspondente à versão. Um repositório privado existente é recusado, preservando sua visibilidade. O workflow **.github/workflows/release.yml** executa os testes no Windows, recompila o ZIP e publica ZIP e SHA-256 em GitHub Releases. O código-fonte também fica disponível no repositório e nos arquivos de fonte da release. Acompanhe a execução em **Actions** e compartilhe o endereço **https://github.com/SUA-CONTA/caderno/releases** após a conclusão. GitHub Actions deve estar habilitado no repositório.

Para uma próxima release, altere a versão em Cargo.toml e atualize CHANGELOG.md; execute novamente publish.ps1. Use uma nova tag para cada versão; o script não sobrescreve tags existentes. A execução manual do workflow gera artefatos para teste; apenas tags publicam uma release.

A pasta dist fica fora do histórico Git; os binários são distribuídos como anexos de Releases. O script usa uma lista explícita dos fontes publicáveis e exclui data, checks, shots e target. Não coloque seus bancos nem registros de alunos nas pastas de código.

## Estrutura

- **src/main.rs**: interface e navegação.
- **src/model.rs**: questões, cálculos, validação, importação e sorteio.
- **src/pdf.rs**: exportação de provas e gabaritos.
- **src/updater.rs**: configuração e consulta de releases.
- **scripts/**: compilação portátil e publicação.
- **.github/workflows/**: automação Windows.
- **MECHANICS.md** e **mechanics.json**: regras preservadas do programa fornecido.
- **CREDITS.md**, **LICENSE** e **CHANGELOG.md**: créditos, licença e histórico.

## Referências e limites

[Gradepen](https://gradepen.com/p/) e [Minha Prova](https://www.minhaprova.com.br) servem como referências de produto para banco de questões, variações de prova, cabeçalhos e gabaritos. O Caderno é uma implementação independente, sem vínculo com essas plataformas. Esta versão prioriza o fluxo local simples; não oferece correção óptica, TRI, inteligência artificial, aplicação online nem colaboração em nuvem.

A interface é um redesenho solicitado do fluxo original em Python/Tkinter. Resultados numéricos usam português e duas casas decimais. O sorteio pode repetir valores entre alunos e não promete exclusividade global. O identificador, o banco e a lista permitem reproduzir versões nesta mesma versão do aplicativo; o registro JSON preserva os resultados exatos mesmo após editar o banco.

HTML do Moodle é convertido em texto simples. Imagens externas ausentes são recusadas com aviso. Os PDFs usam Helvetica/Windows-1252 e Symbol para símbolos matemáticos e letras gregas comuns; outros símbolos recebem substituição simples. Não há renderização LaTeX.

## Licença

Código distribuído sob a **licença MIT**, disponível em LICENSE. Dependências mantêm suas próprias licenças; consulte CREDITS.md.
