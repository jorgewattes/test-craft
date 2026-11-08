# Regras observadas e decisões da reconstrução

Alvo: C:/dev/MoodTest/MoodTest.py, SHA256 c2b9ff9719e5ca4b7c837a22da5eae6a3bd2811d9eb17ef6895a730c038f1541.
Triagem: ev_7bd6a7aa9da148d9276f18620034612777b2c20cc6ec2737c6f9b6e694247863.
O alvo é fonte Python. native_functions_list, native_strings_find e native_function_pseudocode foram tentados e recusaram Unknown file magic. Regras lidas no código original; endereços nativos não se aplicam. Nenhum código decompilado foi copiado.

| Regra | Valor | Fonte (função e linha) |
|---|---|---|
| Operadores permitidos | + - * / potência, unários + e - | _eval_ast L125 e _ALLOWED_OPS L99 |
| Funções permitidas | sqrt abs sin cos tan log ln log10 exp | _ALLOWED_FUNCS L110 e _preprocess_expr L160 |
| Constantes | pi, e | _ALLOWED_CONSTS L120 |
| Variáveis no texto | {nome} | _processar_texto L1075 |
| Cálculo no texto | {=expressão} com variáveis {nome} | _replace_calculated_blocks L457 |
| Casas do texto | 2 | _processar_texto L1087 e _replace_calculated_blocks L478 |
| Arredondamento | metade para longe de zero (ROUND_HALF_UP) | formatar_numero_pt L179 |
| Formato brasileiro | milhares com ponto, decimal com vírgula, remove decimais nulos | formatar_numero_pt L179 |
| Valores por variável | escolha aleatória independente do dataset | Dataset.sample L1001 |
| Alternativas | embaralhar e remapear corretas | Question.instantiate L1036 |
| Seleção por aluno | sem repetição dentro da categoria, sorteio por aluno | _selecionar_questoes_por_plano L2024, gerar_provas L2119 |
| Corretas XML | fraction > 0, pode ter múltiplas | _parse_correct_indices L1437 |
| Tipos XML | calculatedmulti, multichoice | carregar_questoes L1454 |
| Papel | A4 (595,276 × 841,890 pontos) | Exam.export_pdf L1115, reportlab A4 |
| Margem | 2 cm (56,693 pontos) | Exam.export_pdf L1120 |
| Saída | PDFs individuais, conjunto de provas, gabarito consolidado | gerar_provas L2097 |

Novas decisões de produto (não valores recuperados do original): banco JSON versionado, editor com intervalos discretos de 0 a 6 casas, validação de todas as combinações até 10000 ou 256 amostras (avisadas), semente reprodutível, gabarito individual e manifesto com os valores efetivamente sorteados. Exemplos originais da reconstrução, sem logos ou textos institucionais originais.
Imagens opcionais do enunciado e das alternativas são armazenadas dentro do banco e incorporadas nos PDFs. Importação XML mantém imagens PNG/JPG incorporadas e recusa referências externas sem arquivo; HTML vira texto simples. Fonte da regra de preservar imagens: MoodleHTMLParser L1163, Question._render_segments L1053. A inclusão foi decidida após core_person_ask retornar a preferência do projeto para o construtor decidir.

A interface foi redesenhada a pedido do usuário, não existe captura visual executada do original. Fonte observada: Tkinter App.__init__ L1584, cores #f5f7fb e #1f6feb. Tema novo verde e neutro é uma decisão de design.
