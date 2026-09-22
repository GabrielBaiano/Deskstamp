# Deskstamp 🛡️✨

**Deskstamp** é um aplicativo de marca d'água desktop de alto desempenho nativo para Linux (especialmente projetado para **Pop!_OS COSMIC** e compositores Wayland).

Inspirado no Deskmark do macOS, o Deskstamp resolve a necessidade de marcas d'água em tempo real durante transmissões ao vivo, gravações de tela (OBS/PipeWire) e proteção contra vazamento de dados confidenciais (NDAs), sem necessidade de renderização em pós-produção.

---

## Destaques de Engenharia

- **100% Nativo no Wayland**: Implementado em Rust com `smithay-client-toolkit` e protocolo `wlr-layer-shell-unstable-v1`.
- **Click-Through Total**: Define `wl_surface.set_input_region` vazia, garantindo que o mouse e o teclado atravessem o overlay sem qualquer latência ou bloqueio de cliques.
- **Zero CPU Idle**: Damage tracking nativo do Wayland. Se a marca d'água for estática, o renderizador desenha apenas 1 vez e dorme (0.0% CPU).
- **Variáveis Dinâmicas**: Suporta substituição de tokens como `{user}`, `{hostname}`, `{date}`, `{time:%H:%M:%S}`.
- **Pronto para Lojas Linux**: Manifesto Flatpak (`io.github.gabrielbaiano.Deskstamp.json`) e metadados AppStream prontos para Flathub e COSMIC App Store.

---

## Como Rodar Localmente

### Pré-requisitos
- Pop!_OS 24.04 COSMIC (ou qualquer ambiente Wayland compatível com layer-shell)
- Rust toolchain (1.80+)

### Compilação
```bash
cargo build --release
```

### Teste Rápido (5 segundos com click-through)
```bash
./target/release/deskstamp test
```

### Iniciar Daemon do Overlay
```bash
./target/release/deskstamp daemon
```

### Gerar Preview em Imagem (PNG)
```bash
./target/release/deskstamp preview output_preview.png
```

### Controle Via Linha de Comando (IPC)
Com o daemon em execução, você pode controlá-lo de qualquer terminal ou atalho de teclado:
```bash
# Alternar entre visível / oculto
deskstamp toggle

# Recarregar configurações do arquivo
deskstamp reload

# Verificar status
deskstamp status
```

---

## Configuração (`~/.config/deskstamp/config.json`)

O arquivo é gerado automaticamente na primeira execução:

```json
{
  "text": "CONFIDENTIAL • {user}@{hostname} • {time:%H:%M:%S}",
  "font_size": 22.0,
  "angle_deg": -25.0,
  "opacity": 0.18,
  "color_rgba": [255, 255, 255, 255],
  "spacing_x": 420.0,
  "spacing_y": 220.0,
  "stagger_offset": 210.0,
  "stroke_width": 1.0,
  "stroke_color_rgba": [0, 0, 0, 180],
  "font_path": null,
  "active": true,
  "update_interval_secs": 1
}
```

---

## Instalação e Empacotamento

### Flatpak
```bash
flatpak-builder --user --install --force-clean build-dir io.github.gabrielbaiano.Deskstamp.json
```

---

## Licença
Distribuído sob licença GPL-3.0-or-later.
