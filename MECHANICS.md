# Regras verificadas no original

## Melhorias de interface solicitadas (09/11/2026)

Mudanças novas de usabilidade: o banco dedica 48% da largura útil à prévia e
o editor 44%, com opção de janela ampliada e rolagem própria. O menu esquerdo
alterna entre 222 e 72 pontos, mostra dicas nos ícones e guarda a preferência
em `menu-recolhido.json` ao fechar.

Categorias usam dropdown com hierarquia e popup para criação. O campo JSON
opcional `category_labels` preserva categorias vazias, inclusive na exportação
e importação; bancos antigos continuam compatíveis. Caminhos separados por `/`
representam subcategorias; pais implícitos são incluídos e o filtro do banco
inclui descendentes sem confundir nomes com prefixos semelhantes. O plano de
prova mantém contagens por categoria exata para não duplicar questões.

## Conferência atual: tabelas de respostas (09/11/2026)

O principal original `C:/dev/MoodTest/MoodTest.py` foi reaberto com
`core_target_open`. É fonte Python de 83.212 bytes, SHA-256
`c2b9ff9719e5ca4b7c837a22da5eae6a3bd2811d9eb17ef6895a730c038f1541`.
Nesta execução foram aguardadas sequencialmente 3 buscas `native_strings_find`
para nomes, configurações e mensagens, 5 buscas de referências por nome e 22
consultas `native_function_pseudocode`. Todas recusaram o formato com
`not_supported: Unknown file magic`. O registro está em
[checks/native-tables-review.json](checks/native-tables-review.json).
**Não foi obtido pseudocódigo nativo nem endereço de função nativa.** Os locais
abaixo são offsets dos bytes do fonte; `native_address` continua `null`.

Foi seguida a chamada de `Exam._story` (linha 1107, chamada em `0x947b`) para
`add_blank_answer_tables` (linha 968, definição em `0x7e27`). O bloco de respostas
da reconstrução era uma linha de texto; agora usa as regras recuperadas:

| Local no fonte original | Valor, fórmula ou regra | Aplicação na reconstrução |
|---|---|---|
| `add_blank_answer_tables`, `0x7e27`; constante `block = 15` em `0x7f2f` | `start = 1`; `end = min(start + 15 - 1, num_q)`; próximo início `end + 1`. | `src/pdf.rs`, `ANSWER_TABLE_BLOCK = 15`; cada bloco tem no máximo 15 colunas. |
| Mesma função, linhas 976–978 | Cabeçalhos `Q{int(i):02d}` e segunda linha vazia. | Q01 até Q99, depois Q100; campo vazio para cada resposta. |
| Mesma função, linhas 981–984 | Grade preta de 0,75 ponto, cabeçalho cinza claro, texto centralizado em negrito. | Grade e cabeçalho desenhados no PDF; tabela antes das questões nos arquivos individual e combinado. |
| Mesma função, linhas 988–990 | Espaçador de 8 pontos após cada bloco e mais 8 ao final. | Espaçamento de 8 pontos, sem separar o cabeçalho de sua linha de respostas numa quebra de página. |

Evidências dos locais: definição
`ev_86c7eb1d3a69b7330d33e65d872277efea2d4d92677759341d8d648c801819ca`,
constante `ev_8d8b9d3bc7f2e99d5c2a3d8660f555b41b563d62c68754d227154e20e3c3e7bf`,
chamada `ev_a23a9128616ce779568c527ec356339e1bdaec363167cf1ab558dddee49ce18b`.
Foi executada somente essa definição AST isolada com adaptadores de captura de
tabelas, sem importar o aplicativo. Os seis casos de 0, 1, 15, 16, 31 e 100
questões estão em `mechanics.json`, `original.blank_answer_table_cases`, e em
[checks/source-answer-tables.json](checks/source-answer-tables.json).

O teste `blank_answer_tables_match_original` demonstrou a diferença antes da
correção: Q01 estava ausente. Agora compara todos os rótulos com os resultados
do original e verifica uma quebra de página forçada entre Q15 e Q16. A largura
das células e as alturas das linhas pertencem à diagramação própria da
reconstrução; não reproduzem o cálculo automático de dimensões do ReportLab.
O teste de PDF combinado confere a soma real das páginas individuais, pois
incluir a tabela pode aumentar o número de páginas. As demais regras, fórmulas
e limitações recuperadas estão documentadas nas seções seguintes.

Verificação desta execução: **21 testes passaram** em `cargo test --locked`;
`cargo build --release --locked` e o executável release com `--self-check`
concluíram com sucesso. `rebuild_check` passou em compilação, inicialização e
captura (2070 × 1350) e mecânicas (16 valores, 13 com fonte, 3 decisões novas).
A extração dos PDFs confirmou os mesmos grupos de cabeçalhos do original e
linhas de respostas vazias nos seis tamanhos. Todos os PDFs de comparação foram
renderizados; as primeiras páginas de 1, 31 e 100 questões foram inspecionadas
visualmente. Resultados em
[checks/answer-tables-verification.json](checks/answer-tables-verification.json)
e [checks/tables-build-verification.json](checks/tables-build-verification.json).
O executável atualizado é `target/release/test-craft.exe`.

## Conferência atual: propagação de erros de cálculo (09/11/2026)

O programa principal `C:/dev/MoodTest/MoodTest.py` foi novamente aberto com
`core_target_open`; `core_session_show` confirmou Deep mode. Foram aguardados,
em sequência, os resultados de 3 buscas `native_strings_find` (nomes,
configurações e mensagens), 4 consultas de referências por nome e 21 chamadas
`native_function_pseudocode`, cobrindo todas as funções da tabela abaixo.
Todos recusaram o formato Python com `not_supported: Unknown file magic`.
O registro completo desta tentativa está em
[checks/deep-review-latest.json](checks/deep-review-latest.json).
Não houve descompilação nativa bem-sucedida: os locais recuperados são offsets
do arquivo fonte, e `native_address` permanece `null`.

Seguindo `_replace_calculated_blocks` → `avaliar_expressao_com_variaveis` →
`avaliar_expressao` → `_eval_ast` no fonte legível, foi encontrada e corrigida
uma diferença adicional em `src/model.rs`:

| Função / localização real no fonte | Regra recuperada | Resultado observado no original |
|---|---|---|
| `_eval_ast`, `0xc7b`, linha 125; chamada matemática `0x1112` | Erros de domínio e estouro de `math` interrompem a avaliação da subexpressão. Uma potência externa de expoente zero não apaga o erro anterior. | `sqrt(-1)^0` e `log(0)^0`: `ValueError`; `exp(1000)^0`: `OverflowError`. |
| `_eval_ast`, `0xc7b`; operador binário `0xea4` | Zero elevado a expoente negativo gera erro imediatamente. | `(0^-1)^0`: `ZeroDivisionError`. |
| Mesmas funções | Operações válidas continuam disponíveis, inclusive subexpressões acima do limite final da reconstrução. | `sqrt(0)^0 = 1`; `exp(0)^0 = 1`; `(-2)^2 = 4`; `(1e15*10)/10 = 1e15`; `(10^1000)^0 = 1`. |

Evidências dos locais: chamada matemática
`ev_ffa61ec02c7894d1ceff206c11781cbe918dacb184c23518c8d8016a773ca613`,
operador binário
`ev_1f997a123735273ec98d0980b70cc7326b44f5c55017b37c53f2209198e0b04e`.
Os nove resultados foram obtidos executando apenas definições AST isoladas do
original, sem importar o aplicativo, e estão em
[checks/source-domain-reference.json](checks/source-domain-reference.json).
Também foram incorporados a `mechanics.json`, em `original.expression_domain_cases`.
O teste `matches_original_expression_domains` falhou antes da correção porque
`sqrt(-1)^0` retornava `Ok(1.0)`. Agora chamadas matemáticas rejeitam resultados
não finitos imediatamente, e potências verificam zero com expoente negativo.
A decisão de produto de bloquear fórmulas inválidas permanece: o renderizador
original conserva o bloco com erro no texto, conforme descrito abaixo.

As regras já recuperadas continuam sendo usadas: duas casas decimais,
`Decimal(str(x)).quantize(10^-2, ROUND_HALF_UP)`, sorteio independente por entrada
de dataset, remapeamento das alternativas corretas após embaralhamento,
seleção sem reposição por categoria e A4 com margens de 2 cm. A tabela completa
de funções, fórmulas e offsets está abaixo.

Verificação desta conferência: `cargo test --locked` passou com 20 testes;
`cargo build --release --locked` concluiu a compilação otimizada. A nova chamada
`rebuild_check` passou em build, inicialização/captura da interface (2070 × 1350)
e mecânicas (15 valores, 12 com fonte e 3 decisões novas). Os resultados
históricos das seções seguintes pertencem às revisões anteriores.

## Nova conferência solicitada em 09/11/2026

O alvo original foi reaberto com `core_target_open`, em Deep mode. Nesta
conferência foram aguardadas, individualmente e em sequência, **3 buscas
`native_strings_find`** (nomes, configurações e mensagens), **4 consultas
`native_name_xrefs_list`** e **21 chamadas `native_function_pseudocode`**, uma
para cada função da tabela. Todas retornaram `not_supported: Unknown file magic`.
O registro desta execução está em
[checks/native-review-2026-11-09.json](checks/native-review-2026-11-09.json).
O original continua sendo fonte Python; não foi obtido pseudocódigo nativo nem
endereços virtuais. Os locais abaixo são offsets reais no arquivo fonte.

As referências foram seguidas diretamente nas definições e chamadas do fonte.
Os valores e fórmulas recuperados anteriormente foram conferidos novamente:
duas casas, ROUND_HALF_UP decimal, precedência de potências, amostragem por
entrada do dataset, remapeamento das corretas, seleção por categoria e A4/2 cm.
Duas diferenças adicionais foram demonstradas com as funções originais e
corrigidas em `src/model.rs`:

| Função / local no arquivo | Regra recuperada | Aplicação e resultado de referência |
|---|---|---|
| `_eval_ast`, `0xc7b`, linha 125; consulta de contexto `0x118c`, linha 153 | Um nome sem parênteses consulta o dataset antes de `pi` e `e`; chamadas continuam usando as funções permitidas. | Com pi=4, e=2 e sin=5: `{=pi+e+sin} / {pi} / {=sin(pi/2)}` → `11 / 4 / 0,91`. Validação agora permite esses nomes de dataset. Sem dataset, constantes continuam disponíveis. |
| `carregar_questoes`, `0xc549`, linha 1454; escolha de categoria `0xc680`, linha 1463 | A categoria declarada na própria questão é preservada com o caminho completo. | `<category><text>Fisica/Eletricidade</text></category>` → `Fisica/Eletricidade`. A reconstrução continua usando a categoria Moodle corrente quando não há categoria na questão. |

Evidências dos dois trechos, respectivamente:
`ev_bcf0553a6dd1bd086421a3dbdfc6bd6f2241da7bcd4ca9b42b94883acb578927`
e `ev_60b554116c2dcf177e75952fd774504bfab9e32de4ad0632b8931015a3d76bf6`.
Os resultados calculados pelo fonte estão em
[checks/source-review-reference.json](checks/source-review-reference.json).
Foram executadas somente definições AST isoladas; a conferência da categoria
usou adaptadores de texto/imagem, sem importar a GUI ou executar instalação,
rede ou exportação do original. Os testes novos falharam antes das correções,
confirmando as diferenças. O mapa completo de chamadas e offsets permanece
em `mechanics.json`, com `native_address: null` em cada função.

Original: C:/dev/MoodTest/MoodTest.py, 83.212 bytes, SHA-256
c2b9ff9719e5ca4b7c837a22da5eae6a3bd2811d9eb17ef6895a730c038f1541.
Revisão de fonte e nova tentativa de análise nativa realizadas em 09/11/2026.

## Deep mode e limite da descompilação

core_session_show confirmou analysis_mode=deep. O programa principal foi aberto
com core_target_open pelo caminho absoluto acima. É fonte Python legível; não há
um executável nativo original nessa pasta. Não foi descompilado código de máquina.

As chamadas foram aguardadas até o resultado final:

| Ferramenta | Consulta | Resultado |
|---|---|---|
| native_strings_find | Regex de nomes, configurações e mensagens: Dataset, Question, formatar_numero_pt, casas, gerar_provas, Erro, Config | not_supported, Unknown file magic |
| native_name_xrefs_list | Question._processar_texto | Mesmo erro de formato |
| native_function_pseudocode | Cada uma das 21 funções da tabela abaixo, em chamadas individuais e sequenciais | Todas recusadas: not_supported, Unknown file magic |
| native_xrefs_list | Alvo sem endereço nativo | Sem cursor/endereço; não produz referências |

Não há endereços virtuais nem pseudocódigo nativo disponível para este alvo.
Os endereços na tabela abaixo são **offsets de bytes no arquivo fonte**, base zero,
apontando para def, e não endereços de funções em memória. Linhas têm base um.
Forçar o Python como máquina crua produziria uma análise sem relação com o programa.

O registro integral das consultas strings/pseudocode desta revisão está em
[checks/native-analysis-attempts.json](checks/native-analysis-attempts.json).
Cada resultado foi recebido antes da consulta seguinte. Não houve descompilação
nativa bem-sucedida e nenhum endereço virtual foi recuperado.

native_bytes_find funcionou e registrou evidências do arquivo original:

- _processar_texto, 0x8f5a: ev_69316bbeb0878b36d3d7ea5b14f92b95c8cc5ccf022a515bc87548f82fb87c7f.
- _replace_calculated_blocks, 0x3984: ev_2c9e5802f4b6182eefb6017ff3e3a40ed5e283dc212378984934019a29e03915.
- Mensagem "Adicione alunos antes de gerar.", 0x13567, dentro de App.gerar_provas: ev_1866dd2d4c1e6257212d47ff8362289e604806645265653f2906697f8d87eddd.
- Definição e uso de DECIMAL_ZERO_PATTERN, 0x1760 e 0x1859: ev_c204f4d4cdcfc415970a1a9bc6dffc72d430556872d606dca1e1a8a201a66a86.
- _eval_ast, 0xc7b: ev_ba93f19604d9ba865fbf5c403b548069611e08fac90a2cb6b7d05f094b7fa7de.
- Exam.export_pdf, 0x95d0: ev_b0dacbd1a9d68fc9d566d2a6959e8cfadfef8d06c261f647452c5a44542d7c42.
- Ocorrências de fraction (0x85e3, 0xc355, 0xc48a), seguidas até
  _parse_correct_indices e carregar_questoes no fonte:
  ev_1e1acb142f038ffac1d60c92bbf6aa3dbff3d213be0e4cc186da869a9050aab1.

## Funções e regras lidas diretamente

| Função original | Linha | Offset no fonte | Valores, fórmulas e regras |
|---|---:|---|---|
| _eval_ast | 125 | 0xc7b | + - * / **, unários + -; funções sqrt abs sin cos tan log log10 exp; constantes pi, e. log é natural; nomes do contexto precedem constantes. |
| _preprocess_expr | 160 | 0x128e | Remove espaços; troca ^ por ** e ln( por log(. Potência associa à direita: 2**3**2 = 512; -2^2 = -4; 2^-2 = 0,25. |
| avaliar_expressao | 173 | 0x13bd | Faz o parse AST e delega a _eval_ast. |
| formatar_numero_pt | 179 | 0x1501 | Decimal(str(x)).quantize(10^-casas, ROUND_HALF_UP). Milhar . e decimal ,; omite a fração quando o resultado arredondado é inteiro. Ex.: 1.005 → 1,01, -1.005 → -1,01, 1234.5 → 1.234,50 com duas casas. |
| remover_decimais_nulos | 197 | 0x17ac | Regex (-?\d+(?:\.\d{3})*)(?:,0+)(?!\d) remove sufixos decimais totalmente nulos também de números literais. 10,00 → 10; 0,001 permanece. |
| avaliar_expressao_com_variaveis | 219 | 0x1aa2 | Troca vírgula por ponto; substitui {nome} por str(value) **sem parênteses**; também oferece variáveis sem chaves no contexto. Calcula a maior precisão dos datasets usados, padrão 2, mas o renderizador descarta essa precisão e usa sempre 2. |
| _replace_calculated_blocks | 457 | 0x3984 | Encontra {=expressão} respeitando chaves internas; avalia e formata com **2 casas**. Em erro mantém o bloco original. |
| letters_for | 952 | 0x7c98 | Letras minúsculas a, b, c…; após z usa aa, ab… no original. A reconstrução admite até oito alternativas novas. |
| Dataset.sample | 1001 | 0x832c | random.choice(values): escolha uniforme de uma entrada explícita; entradas repetidas mantêm seu peso. |
| Question.instantiate | 1036 | 0x888e | Sorteia cada dataset independentemente, renderiza enunciado e alternativas, embaralha pares (índice_original, alternativa) e remapeia todas as corretas. |
| Question._render_segments | 1053 | 0x8b65 | Processa texto e preserva bytes, dimensões e marcação de falta das imagens. |
| Question._processar_texto | 1075 | 0x8f5a | Remove \( e \); converte \Omega/\ohm em Ω, \omega em ω. Calcula blocos antes de substituir variáveis simples, sempre com 2 casas; limpa decimais nulos e espaços nas extremidades. Comandos como \, permanecem. |
| Exam.export_pdf | 1115 | 0x95d0 | A4: 210/25,4*72 = 595,27559055 × 297/25,4*72 = 841,88976378 pt. Margens dos quatro lados: 2/2,54*72 = 56,69291339 pt. Nome do aluno em cada página. |
| _parse_correct_indices | 1437 | 0xc307 | Aplica strip() à fração; qualquer float(fraction) > 0 indica correta, incluindo 0,5% e 50%; fração inválida vira zero; múltiplas corretas permitidas. |
| carregar_questoes | 1454 | 0xc549 | Tipos calculatedmulti e multichoice; somente calculatedmulti lê datasets. Aplica strip(), troca vírgula decimal por ponto e ignora itens inválidos/vazios; mantém duplicatas e seu peso no sorteio. |
| indices_para_letras | 1518 | 0xd0f7 | Letras minúsculas das corretas separadas por / (a/c); sem correta produz -. Chama letters_for. |
| exportar_gabarito_pdf | 1524 | 0xd1ae | Tabela consolidada, uma linha por aluno e uma coluna por questão; repete o cabeçalho. |
| App.__init__ | 1585 | 0xdaee | Interface original Tkinter. Cores globais L1564–1569: fundo #f5f7fb, cartão #ffffff, destaque #1f6feb, texto #2c3e50, borda #d7deeb, entrada #f9fbff. |
| App._selecionar_questoes_por_plano | 2024 | 0x12aa0 | Quantidade inválida/negativa vira zero; quantidade excessiva é limitada ao estoque com aviso. Mantém ordem das categorias, embaralha dentro delas, sem repetição por aluno. Prioriza questões com imagens completas e usa incompletas como reserva. |
| App._sanitizar_nome | 2091 | 0x1340f | Substitui caracteres inválidos do Windows e espaços por _; nome vazio vira aluno. |
| App.gerar_provas | 2097 | 0x13504 | Exige alunos, questões e plano não vazio. Repete seleção e instanciação para cada aluno; pasta outputl%Y-%m-%d-%H.%M.%S, PDFs individuais, conjunto de provas e gabarito consolidado. |

Referências seguidas no fonte:

~~~
App.gerar_provas → App._selecionar_questoes_por_plano
                → Question.instantiate → Dataset.sample
                                       → Question._render_segments
                                         → Question._processar_texto
                                           → _replace_calculated_blocks
                                             → avaliar_expressao_com_variaveis
                                               → avaliar_expressao
                                                 → _preprocess_expr / _eval_ast
                                           → formatar_numero_pt / remover_decimais_nulos
                → Exam.export_pdf / exportar_gabarito_pdf
carregar_questoes → _parse_correct_indices / Dataset / Question
indices_para_letras → letters_for
~~~

## Aplicação e verificação na reconstrução

src/model.rs usa a precisão recuperada DECIMALS = 2 e a regra decimal ROUND_HALF_UP,
seleção independente de valores, seleção sem reposição por categoria e embaralhamento
preservando corretas. src/pdf.rs usa as fórmulas completas de conversão das
dimensões A4 e margens recuperadas, sem arredondamento antecipado em pontos.

Nesta revisão, render passou a aplicar a normalização de símbolos, delimitadores
matemáticos, espaços e decimais nulos observada no original. eval passou a substituir
variáveis com chaves como texto numérico antes do parse. Isso preserva o resultado
original para negativos: com x = -2, {={x}^2} produz **-4**, {=({x})^2} produz **4**,
e {=x^2} produz **4**. Para elevar o valor negativo completo, use parênteses.

mechanics.json contém o mapa de funções, offsets, referências de chamadas e 14
casos com resultados calculados pelas funções numéricas/textuais do original,
além de três casos de pontuação, quatro casos de letras de gabarito e um caso
de categoria declarado na própria questão.
Os casos foram extraídos das definições numéricas, textuais e de gabarito via AST,
sem importar o módulo inteiro ou executar a instalação do Pillow, GUI, rede ou
exportação do original. O script pessoal de extração foi removido da distribuição.
O teste
matches_original_numeric_text_results compara esses resultados com a implementação
Rust. Os offsets foram calculados sobre os bytes originais, preservando CRLF.

Nesta revisão, a importação passou a aceitar espaços nas pontuações e valores
Moodle, preservar datasets apenas em calculatedmulti e manter entradas repetidas.
O teste moodle_original_dataset_and_score_rules compara a pontuação importada
com resultados de _parse_correct_indices do original e verifica os datasets.
answer_letters_match_original compara o gabarito com indices_para_letras, inclusive
as letras minúsculas e o marcador **-** quando não há corretas. Os testes revelaram
o uso incorreto de maiúsculas; a correção foi aplicada ao editor, prévia, prova e
gabaritos. A geração continua exigindo ao menos
uma correta em questões novas; o marcador também serve a lotes reabertos.

## Decisões de produto e diferenças mantidas

Estas regras são próprias da reconstrução, não recuperadas do original:

- Banco JSON versionado, editor direto com validação, intervalos discretos de 0 a 6
  casas, até 12 variáveis e 2 a 8 alternativas. Até 10.000 combinações são verificadas;
  acima disso usam-se 256 amostras com aviso.
- Fórmulas inválidas, variáveis ausentes, alternativas duplicadas ou resultados não
  finitos impedem a geração. O original preserva blocos de cálculo inválidos no texto.
  Há limites de 512 símbolos, 4.096 caracteres e magnitude de resultado de 1e15.
- Quantidade acima do estoque é recusada para correção explícita do plano; o original
  ajusta automaticamente. Questões com erro, incluindo imagens externas ausentes,
  são importadas e salvas para revisão, com borda vermelha. O XML de cada questão
  é preservado; erros de arquivo XML ilegível continuam sendo informados.
- Sorteio reprodutível por semente com SplitMix64/Fisher–Yates; não reproduz a sequência
  do random do Python. Gabarito individual, CSV e registro JSON guardam o sorteio.
- A importação acompanha categorias Moodle; HTML vira texto simples e imagens
  incorporadas PNG/JPG são preservadas. A diagramação não reproduz todos os segmentos
  HTML do original.
- Limites de 1.000 alunos e 100 questões por prova; nomes repetidos são recusados.
  Dados ficam em data/, em vez da pasta outputl... do original.
- Interface verde/neutra redesenhada a pedido do usuário; sem captura visual do
  original. Banco de exemplos, atualizador GitHub e publicação são recursos novos.

Não se reivindica equivalência nativa nem reprodução integral de todos os detalhes
do original. Para obter pseudocódigo e endereços nativos seria necessário um original
compilado compatível com as ferramentas, que não foi fornecido como alvo.

## Verificação desta revisão (09/11/2026)

- Versão 0.3.0: testes incluem persistência de questões inválidas, importação
  parcial com problemas de conteúdo e detecção de releases MSI.
- As dimensões do PDF usam os resultados completos das conversões A4/2 cm acima,
  também escritos como constantes literais para conferência pelo rebuild_check.
- 19 testes passaram, incluindo os 14 casos numéricos/textuais do original e
  a categoria por questão.
- rebuild_check aprovou build, captura da interface e mecânicas.
- MSI gerado e conteúdo conferido por extração e leitura das tabelas via msi.dll.
  A instalação não foi executada: o serviço Windows Installer está indisponível
  nesta sessão isolada. A validação ICE segue ativa por padrão no workflow;
  a geração local usou a opção explícita -SkipMsiValidation.

Nova execução após as duas correções acima: build release otimizado, MSI e ZIP
gerados novamente. O executável empacotado passou em `--self-check` e coincide
com o binário compilado; os dois checksums SHA-256 foram conferidos. Resultado
em `checks/review-self-check.log`. `rebuild_check` passou em build, inicialização
e captura (2070 × 1350) e mecânicas: 15 valores, 12 com fonte e 3 decisões novas.
A validação ICE/instalação real do MSI continua com a limitação indicada acima.
