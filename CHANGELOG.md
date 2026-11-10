# Test Craft 0.4.2

- Publicação de MSI e ZIP sem assinatura digital, com checksums SHA-256 e validação ICE obrigatória.
- Compatibilidade dos cálculos com o original: literais hexadecimais, octais e binários, separadores numéricos e normalização de `ln(` antes da remoção de espaços.
- Atualização dentro do aplicativo: download de MSI ou ZIP, verificação SHA-256, autorização administrativa do Windows e instalação com preservação dos dados. Falhas e cancelamentos são informados; a versão portátil mantém uma cópia do executável anterior.
- Botões, seletores e dropdowns com medidas, bordas, estados e cores padronizados; seleção de questões com ações em lote consistentes.
- Importação simultânea de arquivos XML/JSON, com conferência única e alteração da categoria por grupo ou por questão antes de confirmar.
- Seleção de várias categorias para mover suas árvores para outra categoria ou apagar categorias e questões, com confirmação e transação atômica.
- Status Pronta/Rascunho no editor e no banco. Provas contam e sorteiam apenas questões prontas e válidas; questões antigas preservam compatibilidade.
- Opção para manter as mesmas questões entre alunos, variando os valores, ou sortear questões independentemente para cada aluno. A opção fica salva nas preferências e no lote.
- Banco otimizado com estatísticas do SQLite, manutenção limitada do WAL, reutilização de contagens e menos consultas e gravações de categorias.

# Test Craft 0.4.1

- Publicação autorizada do MSI e ZIP sem certificado, com checksums SHA-256 e aviso de ausência de assinatura nas notas da release; validação ICE preservada.
- Configuração completa de categorias em uma janela, acessada por ícone; hierarquia sem caracteres ausentes, busca, contagem e exclusão com transferência das questões.
- Conferência antes da importação com totais, erros, distribuição por categoria e escolha de incluir questões que precisam de revisão. XML Moodle mantém os caminhos das subcategorias.
- Cabeçalho de prova renovado com logo opcional; categorias preservadas nas seleções JSON, registros, provas, gabaritos e CSV.
- Ícone original do aplicativo na janela, executável, menu Iniciar e lista de programas instalados.
- Empacotamento preparado para assinatura digital do executável e do instalador, com verificação Authenticode e relatório de assinatura.

# Test Craft 0.4.0

- Banco local em SQLite com índices, gravações incrementais e transações; migração automática do banco JSON, preservando o arquivo anterior e a importação/exportação em JSON.
- Busca em cache, lista com 50 questões por página e validação em segundo plano para reduzir o trabalho repetido em bancos maiores.
- Proteção contra sobrescrita de alterações feitas por outra janela do aplicativo.
- Seleção de questões em lote para exportar, copiar, mover ou apagar, com confirmação de exclusão; edição de categorias e subcategorias.
- PDFs com margens de 1,5 cm, espaçamento compacto, identificação do aluno e rodapé numerado; questões que cabem em uma página ficam junto das alternativas.
- Compatibilidade com o original corrigida na importação de datasets repetidos e nos nomes de arquivos dos alunos.
- Empacotamento Windows compila o SQLite e as demais dependências com a mesma configuração estática de bibliotecas do executável.

# Test Craft 0.3.1

- Prévias maiores no banco e no editor, com janela ampliada; menu recolhível com ícones e preferência salva.
- Categorias e subcategorias em dropdown hierárquico, criação por popup e persistência também de categorias vazias no JSON. Filtro de categoria inclui seus descendentes.
- Provas restauram o gabarito em branco do original: tabelas de até 15 questões, rótulos Q01 em diante e linha para preencher, preservando cada bloco em uma página.
- Cálculos propagam erros de domínio, estouro de funções matemáticas e potência de zero com expoente negativo, conforme o fonte original, mesmo quando uma operação posterior poderia ocultar o erro.
- Atalho de conexão ao GitHub corrigido para pastas com proprietário diferente, sem exceção global de confiança no Git.
- Publicação no GitHub inclui MSI, ZIP e seus checksums SHA-256, com notas apenas da versão atual.
- Conferência adicional do fonte original: datasets podem usar nomes de constantes/funções, com precedência correta; importação preserva a categoria declarada na própria questão.
- Instalador MSI para Windows x64 com atalho no menu Iniciar e atualização preservando dados locais.
- Importação XML preserva questões com erros, imagens inválidas, datasets incompletos e tipos não suportados, mantendo seu XML original para revisão.
- Editor salva questões com erro; borda vermelha e mensagem indicam revisão no banco. Bancos com questões inválidas podem ser reabertos e exportados.
- README resumido e scripts pessoais de publicação, login e inspeção removidos.

# Test Craft 0.2.0

- Compatibilidade Moodle corrigida para espaços em pontuações/datasets, datasets exclusivos de calculatedmulti e gabarito vazio; PDF usa conversões A4/2 cm completas. Revisão Deep documenta a recusa das ferramentas nativas para o original Python e os offsets reais no fonte.
- Aplicativo renomeado para Test Craft na interface, no executável, nos pacotes Windows e nas atualizações; README explica a criação com o Art Craft e como autorizar a publicação no GitHub.
- Regras conferidas no fonte original: símbolos matemáticos, limpeza de decimais e substituição numérica de variáveis; mapa de funções e casos de compatibilidade documentados.
- Área Sobre e atualizações com consulta de releases estáveis do GitHub.
- Verificação automática ao abrir e a cada seis horas, sem bloquear o trabalho.
- Oferta de download quando há versão mais nova com pacote Windows publicado.
- Preferências locais, botão de consulta manual e tratamento de falta de conexão.
- Distribuição Windows x64 portátil, checksum SHA-256, documentação e licença MIT.
- Scripts de compilação/publicação e workflow para releases por tag.

A versão mantém banco local, editor com validação, importação Moodle, provas e gabaritos individualizados em PDF.
