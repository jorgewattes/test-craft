# Test Craft 0.3.0

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
