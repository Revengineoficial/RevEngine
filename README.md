# RevEngine

**RevEngine 0.2.0-pre-alpha.1** — ambiente experimental de criação de jogos 2D/3D para Windows.

> **Aviso:** esta é uma versão pre-alpha destinada a testes iniciais. Podem existir recursos incompletos, falhas e alterações incompatíveis. Faça cópias de segurança dos seus projetos.

## Recursos disponíveis

- Editor desktop nativo em Rust com egui/eframe.
- Viewport 3D, edição e serialização de cenas.
- Geração procedural de cenas e configuração de provedores de IA.
- Integração com Blender por scripts de ponte.
- Ferramentas de importação/exportação para Godot, Roblox, Unity e Unreal.
- Verificador de dependências e utilitários de otimização de assets.

## Executar no Windows

1. Baixe o pacote Windows x64 na seção **Releases** do GitHub.
2. Extraia o ZIP para uma pasta gravável.
3. Execute `RevEngine.exe`.
4. Crie um projeto e salve-o em uma pasta própria.
5. Configure Blender e as credenciais do provedor de IA nas configurações somente se for usar essas integrações.

Para compilar a partir do código-fonte, instale Rust (toolchain stable) e execute:

```powershell
cargo build --release
cargo test --all-targets
```

O executável de desenvolvimento será gerado em `target\release\revengine.exe`.

## Dependências e integrações

- **Blender:** requer instalação compatível do Blender e configuração do caminho do executável.
- **IA:** requer chave de API própria e endpoint/modelo compatível. Nenhuma credencial é distribuída no projeto.
- **Godot, Unity, Unreal e Roblox:** os projetos exportados devem ser abertos e validados na aplicação de destino; a compatibilidade varia conforme a versão e o conteúdo da cena.

## Estado dos testes

Validação local da pre-alpha.1 no Windows x64:

- `cargo check --all-targets` — aprovado.
- `cargo test --all-targets` — 19 testes aprovados.
- `cargo clippy --all-targets` — executado com avisos de estilo não bloqueantes.
- `cargo build --release` — compilação aprovada.

Os testes automatizados não substituem testes manuais da interface, GPU/drivers, Blender ou fluxos completos nas engines externas.

## Relatar problemas

Abra uma issue incluindo versão do Windows, GPU/driver, passos para reproduzir, resultado esperado e resultado observado. Anexe logs quando possível. **Nunca publique chaves de API, senhas ou outros segredos.**

Consulte [RELEASE_NOTES_PRE_ALPHA.md](RELEASE_NOTES_PRE_ALPHA.md) para detalhes e limitações desta versão.
