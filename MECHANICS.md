# Regras verificadas no original

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

mechanics.json contém o mapa de funções, offsets, referências de chamadas e 11
casos com resultados calculados pelas funções numéricas/textuais do original,
além de três casos de pontuação e quatro casos de letras de gabarito.
Gere novamente com:

~~~powershell
python scripts/inspect-original.py C:/dev/MoodTest/MoodTest.py
~~~

O script seleciona as definições numéricas, textuais e de gabarito via AST; não importa o módulo inteiro e
não executa a instalação do Pillow, GUI, rede ou exportação do original. O teste
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
  ajusta automaticamente. Imagens externas ausentes são recusadas na importação;
  o original pode usar questões com imagens faltantes como reserva.
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

- cargo fmt --check e git diff --check passaram; os 16 testes passaram.
- scripts/build-release.ps1 recompilou o executável e o ZIP Windows com sucesso.
- O --self-check do executável empacotado e a conferência SHA-256 do ZIP passaram.
- rebuild_check produziu uma captura atual da interface, conferida visualmente,
  mas a chamada não retornou o relatório: expirou após 300 segundos. Não há
  aprovação completa desse verificador para esta revisão.
