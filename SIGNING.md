# Assinatura do instalador Windows

O MSI e o executável das releases anteriores foram distribuídos sem assinatura digital. Por isso o Windows pode mostrar **Autor desconhecido**. O campo `Manufacturer`, a licença e o checksum SHA-256 não comprovam a identidade do autor.

Para o Windows exibir o autor verificado, é necessário um certificado de **assinatura de código** emitido por uma autoridade confiável, com a identidade do responsável validada. Não serve um certificado de site HTTPS ou um certificado autoassinado. O nome exibido pelo Windows vem do certificado. Não desative o Defender/SmartScreen nem instale uma raiz desconhecida para esconder o aviso.

A versão **0.4.3** será publicada em MSI e ZIP **sem certificado e sem assinatura digital**, mantendo a distribuição da versão 0.4.2 conforme o pedido de publicação. Essa escolha está registrada em `release-config.json` como `unsignedReleaseVersion: "0.4.3"`. O workflow dispensa a importação do certificado somente para essa versão, mantém os testes e a validação ICE do MSI e informa a ausência de assinatura nas notas da release e em `dist/signing-status.json` (`signed: false`). Os arquivos SHA-256 acompanham os dois pacotes. O Windows pode continuar mostrando **Autor desconhecido**.

Para versões sem essa autorização explícita, o processo assina o EXE antes de incluí-lo no ZIP e no MSI, assina o MSI final, usa SHA-256 e carimbo de tempo RFC 3161 e verifica as duas assinaturas com a política Authenticode do Windows. Os checksums são calculados depois da assinatura. Falhas na assinatura, no carimbo de tempo ou na verificação interrompem o empacotamento. O resultado fica em `dist/signing-status.json`.

Mesmo com assinatura válida, um aplicativo novo ainda pode receber um aviso de reputação do SmartScreen. Assinatura confiável identifica o autor; não promete eliminar todo aviso de reputação. Consulte a [explicação da Microsoft sobre SmartScreen](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation) e a [documentação do SignTool](https://learn.microsoft.com/en-us/dotnet/framework/tools/signtool-exe).

## Preparar uma release assinada

Instale o certificado e seu provedor/chave privada conforme as instruções da autoridade emissora, no repositório **Pessoal** do usuário Windows que executa a compilação. Chaves em token precisam do token e do software do fornecedor. Com o Windows SDK instalado, informe o thumbprint público do certificado:

```powershell
$env:TEST_CRAFT_SIGNING_THUMBPRINT = 'THUMBPRINT_DO_CERTIFICADO'
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-release.ps1
```

O script não escolhe uma identidade automaticamente. Para certificados no repositório do computador, use `-SigningCertificateStore LocalMachine`. O caminho do SignTool pode ser definido com `TEST_CRAFT_SIGNTOOL_PATH`; o servidor RFC 3161 pode ser configurado com `-TimestampUrl`. A chave privada não sai do provedor de certificados e nenhuma senha é passada ao SignTool.

Para voltar a publicar com assinatura, remova `unsignedReleaseVersion` de `release-config.json`. No GitHub Actions, configure os secrets do repositório `TEST_CRAFT_SIGNING_PFX_BASE64` (PFX em base64) e `TEST_CRAFT_SIGNING_PFX_PASSWORD` (senha do PFX), somente se o emissor permitir esse formato de chave. O workflow importa a chave temporariamente, compila e verifica os pacotes e remove os certificados importados. Sem esses secrets, versões que exigem assinatura param com uma mensagem explicativa. Certificados que exigem token ou serviço remoto precisam de um executor/provedor compatível; não exporte uma chave que o emissor não permite exportar. Nunca coloque PFX, chave ou senha no código ou no chat.

## Compilação local sem certificado

Sem certificado, ainda é possível conferir o aplicativo e gerar pacotes locais explicitamente sem assinatura:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-release.ps1 -AllowUnsigned
```

Esses pacotes continuam sujeitos ao aviso de autor desconhecido e o relatório informa `signed: false`. Em tags no GitHub, `-AllowUnsigned` sozinho continua bloqueado: a versão também precisa corresponder à autorização em `release-config.json`. A autorização de 0.4.3 não libera versões futuras automaticamente. A validação ICE continua obrigatória nas tags; `-SkipMsiValidation` serve apenas para preparação local onde o serviço Windows Installer não está acessível.
