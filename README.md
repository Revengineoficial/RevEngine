# RevEngine Studio

**RevEngine Studio** é um ambiente experimental de criação de jogos 2D e 3D, desenvolvido em Rust. A proposta é reunir em uma única ferramenta a edição de cenas, a organização de assets, a assistência de IA, a geração procedural, a integração com Blender e a exportação de projetos para diferentes engines.

> **Estado atual: pre-alpha.** O RevEngine ainda está em desenvolvimento. Algumas integrações geram projetos-base ou arquivos de intercâmbio, não jogos finais prontos para publicação. Faça backup dos projetos antes de testar e valide todo conteúdo exportado na engine de destino.

## Objetivo do projeto

A meta do RevEngine é reduzir o trabalho repetitivo do desenvolvimento de jogos: começar por uma ideia, montar uma cena, criar ou importar recursos, organizar scripts e gerar uma estrutura de projeto que possa ser continuada no runtime próprio ou em outra engine.

A longo prazo, a intenção é evoluir para um fluxo integrado no qual o usuário descreve uma mecânica ou cenário, a IA ajuda a planejar e criar os arquivos, o Blender produz modelos e assets, a cena aparece no editor e o projeto pode ser compilado para a plataforma escolhida.

## O que já existe nesta pre-alpha

### Editor e cenas

- Aplicativo desktop para Windows, feito em Rust com egui/eframe.
- Viewport 3D de edição com grade, eixos, bússola, seleção de objetos e controles de câmera.
- Hierarquia de cena e inspetor para editar nome, visibilidade, posição, escala, cor e parâmetros básicos de material e física.
- Objetos iniciais: cubo, esfera, plano, luz e câmera; a cena também reconhece outros tipos usados pelos formatos de intercâmbio.
- Abrir e salvar projetos `.rev` e cenas JSON.
- Navegador de arquivos de assets e editor simples de scripts de texto.
- Console com mensagens de estado, integração e erros.

O viewport do editor é uma visualização leve de trabalho, não um renderizador PBR final. Os controles de material e física representam dados de cena; a aparência e o comportamento finais dependem do runtime ou da engine de destino.

### Runtime próprio e execução

Ao criar um projeto, o RevEngine prepara uma estrutura com `Scenes`, `Scripts`, `Assets`, `Build`, `Plugins`, `Integrations` e `Runtime`. O runtime-base usa **Bevy 0.16** e **bevy_rapier3d 0.30**.

O botão **Play** executa `cargo run --release` dentro da pasta `Runtime`. O template atual carrega uma cena JSON, instancia primitivas e modelos GLB, cria câmera e iluminação e disponibiliza física básica por Rapier. É um ponto de partida experimental, não um runtime completo com todos os sistemas de um jogo comercial.

### RevAI e geração assistida

O painel RevAI permite configurar um provedor, modelo, endpoint e chave de API. Os provedores pré-configurados são NVIDIA, OpenAI, Claude, DeepSeek e um endpoint customizado/local compatível com o formato esperado.

Dependendo do provedor e da resposta, o assistente pode:

- interpretar uma solicitação de criação ou alteração de jogo;
- gerar arquivos e dados de cena em JSON;
- atualizar a cena do editor quando a resposta contém uma cena reconhecida;
- acionar geração procedural para alguns tipos de cenário;
- enviar ações de modelagem para o Blender quando o caminho do executável estiver configurado;
- acionar exportações adicionais quando a solicitação pede explicitamente formatos ou engines compatíveis.

A IA depende de conexão com a internet ou de um serviço local em execução, de uma chave válida quando exigida e de um modelo/endpoint compatível. As respostas geradas devem ser revisadas. Não há modelo de IA incluído no aplicativo.

**Atenção às chaves:** as configurações são gravadas localmente em `%APPDATA%\RevEngine\settings.json`. Nesta versão, as chaves ficam nesse arquivo de perfil e não há cofre criptográfico integrado. Não compartilhe esse arquivo nem publique capturas que revelem credenciais.

### Blender

A integração com Blender foi ampliada para trabalhar com malhas de alta densidade e um fluxo de assets mais automático. A ponte Python aceita geração de primitivas com subdivisão, bevel, normais ponderadas e UVs automáticos; terrenos procedurais com múltiplas frequências e até 512 × 512 células; ambientes com biomas, dispersão de árvores e rochas; importação de GLB/glTF, FBX e OBJ preservando materiais e animações suportadas pelo formato; materiais PBR com albedo, roughness, metallic e normal; e exportação GLB junto da fonte editável .blend e render de prévia. Há presets de animação procedural (idle, walk, run, jump e attack) para objetos e armatures reconhecidos; a qualidade do retarget depende dos nomes e hierarquia dos ossos do rig. As ações podem ser solicitadas ao RevAI usando `generate_environment` e `create_animation_preset`. É necessário ter Blender instalado e configurar o caminho de `blender.exe` em **Configurações → Ferramentas & Caminhos**.

O Content Browser inclui uma **Biblioteca Gratuita 3D / PBR** ligada ao catálogo da Poly Haven. A busca consulta modelos, texturas e HDRIs; os downloads são organizados em `Assets/Models`, `Assets/Textures` e `Assets/HDRI`, com `LICENSE.txt` por asset. Modelos GLTF/GLB são inseridos automaticamente na cena com referência ao arquivo e seus materiais/dependências. Para texturas, a integração baixa os mapas PBR disponíveis (difuso/albedo, roughness, metalness, normal, AO e displacement quando publicados) e aplica automaticamente o mapa de cor ao objeto selecionado ou ao último modelo da cena; os mapas adicionais permanecem no projeto para configuração avançada no Blender. HDRIs são baixados para uso na iluminação. No Blender, assets importados recebem UVs automáticos quando não possuem UV map e material PBR padrão quando chegam sem material. Animações embutidas em GLB/FBX compatíveis são preservadas pelo importador do Blender. A biblioteca Poly Haven não oferece catálogo de animações. Para movimentos prontos gratuitos, o fluxo documenta a **Quaternius Universal Animation Library**, com mais de 120 clipes humanoides sob CC0; baixe o pacote Standard gratuito na página oficial e importe os GLB/FBX pelo Content Browser. O retarget procedural do RevEngine ainda é básico e depende da compatibilidade do esqueleto; não há download automático direto do itch.io nesta versão.

A biblioteca usa a API pública da Poly Haven e identifica os recursos como CC0 conforme declarado pelo catálogo; a licença e a página do recurso devem ser conferidas antes de redistribuição. A integração é executada em modo background. Ela não substitui o Blender completo e não garante conversão perfeita de qualquer geometria.

### Importação e exportação

O menu **Integrações** contém geradores e conversores experimentais para:

| Destino/formato | O que o RevEngine gera ou importa nesta versão |
|---|---|
| Blender | Script Python de cena e exportação GLB; importa `.blend` copiando o arquivo para Assets |
| Godot 4.x | Estrutura de projeto e cena `.tscn`; importação básica de cenas textuais `.tscn` |
| Unity | Estrutura de projeto, cena YAML e prefabs básicos; importação textual de `.unity` e `.prefab` |
| Unreal Engine 5 | Arquivo `.uproject` e scripts de automação; requer validação e execução no Unreal Editor |
| Roblox Studio | Place `.rbxlx`, estrutura Rojo e scripts Luau básicos; importação textual de `.rbxlx` |
| GLB, glTF, OBJ, FBX | Copia o modelo para `Assets/Models` e adiciona uma referência de modelo à cena |
| PNG, JPG, JPEG, TGA, BMP | Copia a textura para `Assets/Textures` |
| Python, Lua e Luau | Copia scripts para as pastas de scripts do projeto |

A opção de exportação universal tenta gerar saídas para Roblox, Godot, Unity, Unreal, C++/CMake, C# e Lua. O sucesso da geração de arquivos não significa que todos esses projetos já estejam compilados ou validados pelos respectivos fabricantes. Alguns destinos são esqueletos iniciais e precisam de dependências, código adicional e testes dentro da ferramenta de destino.

## O que o RevEngine pretende permitir nos jogos

A visão do projeto é oferecer um fluxo progressivamente mais completo para:

1. **Prototipar jogos 2D e 3D:** criar cenas, posicionar objetos, organizar câmeras, luzes e elementos de cenário.
2. **Construir ambientes:** usar geração procedural e Blender para criar terrenos, estruturas, props e modelos 3D.
3. **Preparar personagens e objetos:** organizar modelos, materiais, colisores e dados de transformação.
4. **Criar mecânicas:** gerar e editar scripts, lógica de jogo, interações e sistemas de gameplay com assistência de IA.
5. **Testar e iterar:** executar o runtime-base, observar a cena, corrigir recursos e repetir o ciclo de criação.
6. **Levar o projeto adiante:** exportar estruturas para engines externas e, conforme a integração amadurecer, automatizar builds para as plataformas suportadas.

Os itens acima descrevem o objetivo de evolução. Recursos como editor visual completo de animação, sistema de áudio, multiplayer, editor de código com depuração, pipeline PBR completo, empacotamento multiplataforma e builds finais automatizados ainda não devem ser considerados concluídos nesta pre-alpha.

## Estrutura de pastas

```text
RevEngine/
├── assets/                 # Recursos visuais da própria ferramenta
├── blender/                # Scripts e ponte de automação do Blender
├── examples/               # Exemplos
├── scripts/                # Scripts auxiliares e instalação de ferramentas
├── src/                    # Código Rust do editor e integrações
├── templates/              # Templates de projetos e runtime
├── tools/                  # Ferramentas auxiliares; Godot pode estar incluído
├── Cargo.toml              # Manifesto do editor
├── Cargo.lock              # Dependências travadas do editor
└── README.md
```

## Requisitos no Windows

- Windows 10 ou Windows 11 de 64 bits.
- Rust stable com Cargo.
- Microsoft C++ Build Tools (MSVC) e Windows SDK para compilar dependências nativas.
- Conexão com a internet na primeira compilação para baixar crates e dependências.
- Blender instalado para usar a ponte de modelagem/exportação.
- Python 3 e os pacotes `trimesh`, `Pillow` e `numpy` para alguns utilitários auxiliares.
- Godot, Unity, Unreal ou Roblox Studio instalados separadamente para abrir e validar os projetos exportados correspondentes.
- Chave de API ou endpoint local compatível para utilizar o RevAI.

## Compilar no seu PC

Abra PowerShell na pasta do repositório:

```powershell
cd "$env:USERPROFILE\OneDrive\Área de Trabalho\RevEngine"

# Conferir a formatação
cargo fmt --all -- --check

# Executar os testes automatizados
cargo test --all-targets

# Compilar o editor otimizado
cargo build --release

# Abrir o editor
.\target\release\revengine.exe
```

O executável do editor é criado em `target\release\revengine.exe`. Se o Windows não encontrar o linker, instale o workload **Desenvolvimento para desktop com C++** no Visual Studio Build Tools e tente novamente.

### Executar/compilar um jogo criado

Depois de criar ou abrir um projeto no RevEngine, use o botão **Play** ou execute manualmente:

```powershell
cd "CAMINHO\DO\PROJETO\Runtime"
cargo run --release
```

Para gerar o executável do runtime-base:

```powershell
cargo build --release
```

O binário gerado fica em `Runtime\target\release\revengine_game.exe` no Windows. A primeira compilação do runtime pode demorar porque o Bevy e suas dependências precisam ser baixados e compilados. Os arquivos de cena usados pelo template ficam em `Scenes\Main.scene.json` na raiz do projeto.

## Testes e diagnóstico

- `cargo fmt --all -- --check`: verifica a formatação Rust.
- `cargo check --all-targets`: verifica o código e os alvos de teste.
- `cargo test --all-targets`: executa os testes automatizados.
- `cargo clippy --all-targets`: aponta problemas e sugestões estáticas.
- `cargo build --release`: compila o editor em modo otimizado.
- O botão **Doctor** verifica Rust/Cargo, Python, Blender, Godot e pacotes Python.
- O **Console & Logs** exibe mensagens do editor e das integrações.

Na revisão local desta pre-alpha, `cargo check --all-targets` passou e `cargo test --all-targets` aprovou 21 testes. A suíte incluiu um teste headless real no Blender 5.3.0: a cena de exemplo foi exportada para GLB e renderizada em PNG, e os dois arquivos foram verificados. O Clippy ainda aponta 30 avisos de estilo não bloqueantes (principalmente simplificações de `if`); eles não equivalem a falhas de compilação. O editor foi aberto no Windows e a tela principal, o viewport 3D, a hierarquia e os painéis de IA/assets foram inspecionados visualmente. Isso não certifica individualmente todos os botões e diálogos.

Os testes automatizados não substituem o teste manual de cada controle, GPU/driver, diálogos do Windows, serviços de IA, Blender instalado e projetos abertos em engines externas.

## Limitações conhecidas da pre-alpha

- O viewport é uma representação de edição simplificada; não é um renderizador completo de jogos.
- O runtime Bevy/Rapier é um template inicial. Nem todos os campos de cena e propriedades do inspetor já são reproduzidos pelo runtime.
- Algumas exportações são estruturas iniciais, não builds finais; valide os arquivos na versão instalada da engine de destino.
- A importação de formatos 3D copia o arquivo e cria uma referência na cena; não há garantia de leitura de toda geometria/material no viewport.
- A integração Blender exige instalação externa e pode falhar se o executável, add-on GLB ou versão não forem compatíveis.
- RevAI pode demorar, falhar por limite/autenticação/rede ou produzir arquivos que precisem de revisão.
- O salvamento de chaves em `settings.json` não é criptografado nesta versão.
- A otimização automática é heurística e deve ser revisada antes de usar em projetos grandes.

## Como relatar um bug

Ao abrir uma issue, informe:

- versão do RevEngine e do Windows;
- CPU, GPU e versão do driver gráfico;
- passos exatos para reproduzir o problema;
- resultado esperado e resultado observado;
- mensagens relevantes do Console & Logs;
- versões do Blender ou da engine externa, quando aplicável.

Remova chaves de API, senhas, tokens e dados pessoais de logs e capturas antes de compartilhar.

## Licença e estado do projeto

RevEngine está em desenvolvimento pre-alpha. Consulte os arquivos de licença e avisos de terceiros presentes no repositório antes de redistribuir builds ou dependências.
