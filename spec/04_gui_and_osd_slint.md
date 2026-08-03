# Spec 04: Componentes Gráficos e OSDs em Slint (`sway-manager-gui`) 🎨

## 1. Responsabilidade da Crate
Implementa todas as interfaces gráficas e popups OSD utilizando o framework **Slint** (renderização nativa Wayland com aceleração via GPU e baixíssimo consumo de memória RAM, ~3MB por janela).

---

## 2. Componentes Slint a Serem Desenvolvidos

### A. Popups OSD (Volume & Brilho)
- **Design**: Card moderno arredondado com cantos suavizados e barra de progresso suave.
- **Comportamento**: Exibição centralizada na tela durante 1.2 segundos, fechando automaticamente sem reter memória em RAM.

```slint
// Exemplo de componente Slint para OSD
export component OsdWindow inherits Window {
    in property <string> icon: "🔊";
    in property <int> value: 50;
    in property <string> title: "Volume";

    width: 240px;
    height: 140px;
    background: #1e1e2e;
    no-frame: true;

    VerticalLayout {
        padding: 16px;
        alignment: center;
        Text { text: icon; font-size: 32px; horizontal-alignment: center; }
        Text { text: title; font-size: 14px; color: #cdd6f4; horizontal-alignment: center; }
        Rectangle {
            height: 8px;
            background: #313244;
            border-radius: 4px;
            Rectangle {
                width: parent.width * (value / 100);
                background: #89b4fa;
                border-radius: 4px;
            }
        }
    }
}
```

### B. Seletor de Papéis de Parede (`WallpaperPickerWindow`)
- **Grid de Imagens**: Exibe miniaturas em grade (240x135 px) utilizando a crate `image` para decodificar diretamente nas dimensões da miniatura.
- **Teclado & Atalhos**: Navegação por setas do teclado, tecla `Enter` para aplicar e `Escape` para fechar.

### C. Central de Controle / Settings (`ConfigCenterWindow`)
- **Visual**: Interface no estilo macOS com painel lateral esquerdo (Sidebar) e abas:
  - **Bateria & Energia**: Conservação de Bateria, Inatividade, Perfil de Energia.
  - **Exibição**: Modos de Monitor (Dual, PC Only, External).
  - **Aparência**: Tema Escuro/Claro, Pasta de Wallpapers e Screenshots.
  - **LightDM**: Configurações de Login e Imagem de Fundo.

### D. Overlay de Seleção de Screenshot (`FreezeSelectionOverlay`)
- Janela em tela cheia transparente Wayland para recorte de área com o mouse.
