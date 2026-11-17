# RevEngine 0.2.0-pre-alpha.1

**Canal:** pre-alpha / testes iniciais
**Plataforma compilada:** Windows x64
**Aviso:** build experimental. Faça backup dos projetos antes de testar.

## O que há nesta build

- Editor desktop em Rust/egui com viewport 3D, hierarquia, inspetor, console e navegador de assets.
- Projetos `.rev`, cenas JSON e estrutura de projeto com runtime Bevy 0.16 + Rapier 3D.
- RevAI com provedores configuráveis e geração assistida de arquivos/cenas.
- Ponte Blender com malhas densas, terrenos multi-octave até 512×512, biomas com dispersão de vegetação/rochas, materiais PBR, UV automático, presets de animação procedural (idle/walk/run/jump/attack), importação GLB/GLTF/FBX/OBJ, fonte .blend e exportação GLB.
- Biblioteca Poly Haven para modelos, texturas PBR e HDRIs gratuitos CC0 declarados, com download organizado por projeto e aplicação automática do mapa de cor.
- Link integrado para a Quaternius Universal Animation Library (120+ animações CC0); o download do pack é feito na página oficial e a importação no engine é local.
- Exportadores/conversores experimentais para Godot, Unity, Unreal, Roblox, C++, C# e Lua.

## Correções incluídas nesta revisão

- Encerramento do processo do runtime e dos processos filhos ao usar Stop; detecção de término espontâneo.
- Carregamento da cena de inicialização indicada no projeto e aviso visível quando a cena falha ao abrir.
- Busca de assets funcional e erros de importação apresentados no console.
- Salvamento de configurações com tratamento de erros real.
- Escape seguro de nomes ao gerar scripts Python para Blender.
- Prévia Blender realmente carregada como textura no viewport.
- Descoberta da ponte Blender relativa ao executável, para builds distribuídas com a pasta `blender`.
- Blender executado sem pipe de stderr bloqueante; falhas do processo são comunicadas ao RevAI.
- Render de prévia usa Cycles em CPU para evitar a falha Vulkan/EEVEE observada neste PC.

## Validação nesta máquina

- `cargo fmt --all` — aplicado.
- `cargo check --all-targets` — aprovado.
- `cargo test --all-targets` — 21 testes aprovados.
- Blender 5.3.0 — teste headless de terreno procedural (9.409 vértices), dispersão de árvores/rochas, preset de animação walk e exportação GLB + .blend aprovado.
- Correção de compatibilidade Blender 5.x: ações não acessam mais `Action.fcurves` legado; planos JSON com BOM UTF-8 também são aceitos.
- `cargo clippy --all-targets` — executado; ainda há avisos de estilo não bloqueantes.
- `cargo build --release` — build Windows x64 otimizado concluído.

A interface foi aberta e inspecionada visualmente. Não foi possível certificar individualmente todos os botões, diálogos, provedores de IA ou engines externas. Teste cada fluxo no seu ambiente antes de usar em produção.

## Como iniciar

1. Extraia o ZIP inteiro para uma pasta gravável.
2. Mantenha a pasta `blender` ao lado de `RevEngine.exe` para habilitar a ponte do Blender.
3. Execute `Start RevEngine.bat` ou `RevEngine.exe`.
4. Configure Blender e o provedor de IA em Configurações se for usar esses recursos.
5. Crie um projeto e teste os fluxos com cópias dos seus arquivos.

## Limitações conhecidas

- O viewport do editor é uma visualização simplificada, não um renderizador PBR final.
- O runtime Bevy/Rapier é um template inicial e ainda não reproduz todas as propriedades do editor.
- Exportações para engines externas podem exigir ajustes manuais, plugins e validação na versão instalada.
- A importação de modelos 3D copia o arquivo e cria uma referência; não garante importação completa de geometria e materiais para o viewport.
- As chaves de API ficam em `%APPDATA%\RevEngine\settings.json` sem criptografia integrada nesta pre-alpha. Proteja esse arquivo.
- Recursos de IA dependem de rede, credenciais e limites do provedor; revise os arquivos gerados.

## Relato de bugs

Inclua Windows, CPU/GPU, driver, passos para reproduzir e logs relevantes. Remova senhas, tokens e chaves de API antes de compartilhar qualquer arquivo ou captura.
