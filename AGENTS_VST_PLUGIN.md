# BeatPartner VST Plugin — AI Agent Instructies

## Project Context

BeatPartner Plugin is een **standalone VST3 / AU / CLAP plugin** (geschreven in Rust) die muziekproducers begeleidt binnen hun DAW. In tegenstelling tot de desktop companion app draait deze **direct in de DAW** en heeft toegang tot real-time audio. Er is **geen AI chatbot** — alle features zijn deterministisch, tool-based of opgeslagen gebruikerscontent.

## Architectuur

- **Plugin Framework**: Rust + [nih-plug](https://github.com/robbert-vdh/nih-plug) (VST3 + AudioUnit + CLAP)
- **UI**: egui (Rust immediate mode GUI) — geen webview, geen React
- **Audio Analyse**: Real-time buffer processing via symphonia + realfft
- **Database**: SQLite (rusqlite) voor settings, lyrics, annotations, project notes
- **Styling**: Donker thema, glasmorfisme-achtige egui-styling

## Code Conventies

### Taal

- **Code**: Engels (variabelen, functies, comments, commit messages)
- **Documentatie**: Nederlands (PROJECT_PLAN_VST_PLUGIN.md)
- **UI teksten**: Engels (interface labels, toasts, dialogs)

### Rust

- **Strict real-time safety** — nooit alloceren, locken, of file/DB I/O in de audio callback (`process()`)
- Gebruik `thiserror` voor error types, `anyhow` alleen in build-tools/niet-realtime code
- Business logic gescheiden van plugin lifecycle code
- Data models in `src/models/` met `serde::Serialize` + `serde::Deserialize`
- Alle structs die naar de UI worden gestuurd MOETEN `#[serde(rename_all = "camelCase")]` hebben
- Enums als string waarden gebruiken `#[serde(rename_all = "kebab-case")]`

### Real-Time Safety (CRITICAAL)

De audio callback (`process()`) wordt aangeroepen elke ~1-10ms. **Regels:**

1. ❌ **GEEN allocatie** (`String::new()`, `vec![]`, `Box::new()` na init)
2. ❌ **GEEN blocking I/O** (file reads, SQLite queries, network)
3. ❌ **GEEN mutex locks** (geen `std::sync::Mutex` — gebruik `lockfree` of crossbeam kanalen)
4. ❌ **GEEN logging in audio thread** (geen `println!` of `eprintln!`)
5. ✅ **Wel**: pre-allocation bij `initialize()`, ring buffers, atomic types, lock-free queues

**Patroon voor zware berekeningen:**

```rust
// In audio callback: schrijf audio naar ring buffer, NIETS meer
fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, _context: &mut impl ProcessContext<Self>) -> ProcessStatus {
    self.audio_ring_buffer.write(buffer);
    ProcessStatus::Normal
}

// In background thread (NIET audio thread):
// - Lees uit ring buffer
// - Voer FFT / BPM detectie uit
// - Schrijf resultaat naar SQLite
// - Stuur resultaat terug naar UI via crossbeam channel
```

## Bestandsstructuur

```
├── src/
│   ├── lib.rs               # Plugin entry point + nih-plug wrapper
│   ├── editor.rs            # egui editor setup
│   ├── params.rs            # Plugin parameters (automation-exposed)
│   ├── audio/               # Real-time audio code
│   │   ├── processor.rs     # Audio callback (strict real-time)
│   │   ├── ring_buffer.rs   # Lock-free ring buffer
│   │   └── detector.rs      # BPM/Key detection algorithms
│   ├── ui/                  # egui UI code
│   │   ├── app.rs           # Hoofd UI state + tab router
│   │   ├── theme.rs         # Donker thema + kleuren constants
│   │   ├── widgets/         # Herbruikbare egui widgets
│   │   ├── lyrics/          # Lyrics editor + annotaties
│   │   ├── vocals/          # Vocal production panels
│   │   ├── theory/          # Theory helper (chords, scales)
│   │   ├── analyzer/        # Spectrum / audio visualizer
│   │   └── settings.rs      # Plugin settings panel
│   ├── services/            # Business logic (NIET real-time)
│   │   ├── db_service.rs    # SQLite database access
│   │   ├── analysis_service.rs  # Audio analyse (background thread)
│   │   └── project_service.rs   # Project notes / metadata
│   ├── models/              # Data structs
│   │   ├── project.rs
│   │   ├── lyrics.rs
│   │   ├── vocal.rs
│   │   └── analysis.rs
│   └── error.rs             # BeatPartnerError enum
├── assets/
│   └── fonts/               # Custom fonts voor egui
├── migrations/              # SQLite migratie SQL bestanden
└── Cargo.toml
```

## Design System — Donker & Glasmorfisme

### Kleuren Schema

```rust
// ui/theme.rs
pub const SURFACE_PRIMARY: Color32 = Color32::from_rgb(0x02, 0x02, 0x04);
pub const SURFACE_SECONDARY: Color32 = Color32::from_rgb(0x08, 0x08, 0x0c);
pub const SURFACE_TERTIARY: Color32 = Color32::from_rgb(0x0f, 0x0f, 0x14);
pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0x22, 0xd3, 0xee);
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(0xa7, 0x8b, 0xfa);
pub const ACCENT_MAGENTA: Color32 = Color32::from_rgb(0xf4, 0x72, 0xb6);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xe0, 0xe0, 0xe0);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x99, 0x99, 0x99);
pub const BORDER: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 30);
pub const GLASS_BG: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 20);
```

### UI Patterns in egui

```rust
// Container met glas-effect
fn glass_panel(ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
    Frame::none()
        .fill(GLASS_BG)
        .rounding(12.0)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(16.0)
        .show(ui, content);
}

// Interactieve knop
fn glass_button(ui: &mut Ui, label: &str) -> Response {
    ui.add(
        Button::new(RichText::new(label).color(TEXT_PRIMARY))
            .fill(SURFACE_SECONDARY)
            .rounding(8.0)
            .stroke(Stroke::new(1.0, BORDER))
    )
}
```

### Layout

- Sidebar links met icons + labels (Guides, Lyrics, Theory, Analyzer, Settings)
- Main content rechts van sidebar
- Status bar onderaan met huidige BPM / Key / Detectie status
- Standaard plugin grootte: 900×600 (resizeable via DAW)

### Typografie

- **Labels**: 11px, TEXT_SECONDARY
- **Body**: 13px, TEXT_PRIMARY
- **Headings**: 15px, medium weight
- **Monospace**: Altijd voor data (BPM, key, timestamps)

## Belangrijke Regels

1. **Geen AI chat** — dit is een tool/plugin, geen chatbot. Alle features zijn deterministisch.
2. **Audio thread = heilig** — nooit DB I/O, allocatie, of blocking calls in `process()`.
3. **SQLite alleen vanuit background thread** — gebruik `std::thread` of `tokio::runtime` (single-threaded) voor DB operaties. Communicatie via channels.
4. **currentProject leeft in DB** — niet in globale state. Laad bij editor open.
5. **Audio analyse resultaten cachen** — geen herhaalde FFT op dezelfde audio. Cache in SQLite (`audio_analysis` tabel).
6. **Plugin state = presets** — gebruik nih-plug's `Params` systeem voor settings die de DAW moet kunnen automatiseren/opslaan.
7. **Migraties zijn forward-only** — geen rollbacks.
8. **Test op alle DAWs** — Logic Pro, Ableton Live, FL Studio, Reaper. Gedrag verschilt per host.

## Vocal & Lyrics Conventies

### Vocal Scope

**Vocal Assistant, geen DAW**: BeatPartner is een companion, geen DAW. Vocal features bevatten géén eigen audio opname. De app helpt de producer met guides, checklists, notities en advies om vocals in hun eigen DAW te realiseren.

### Lyrics Annotatie Conventie

**Lyrics Indexering**: Annotaties gebruiken UTF-16 code-unit offsets (`start_index`, `end_index`) om consistentie met de UI text engine te garanderen.

**Tag Kleuren**:

- `melody` → accent-cyan
- `ad-lib` → accent-magenta
- `harmony` → accent-purple
- `flow` → Color32::from_rgb(0xf5, 0xb0, 0x40)
- `emphasis` → Color32::from_rgb(0xfb, 0x71, 0x85)
- `note` → TEXT_SECONDARY

### Reference Vocal Conventie

**Reference Vocals**: Geïmporteerde vocal tracks worden behandeld als referentie-audio. Ze worden alleen geanalyseerd voor metadata (BPM/key/duration) en opgeslagen als samples. Geen editing/bewerking van de audio zelf.

## Database Conventies

- Enkele SQLite connectie met WAL mode
- DB operaties lopen in een dedicated background thread
- Gebruik crossbeam channels voor communicatie audio thread ↔ background thread
- Caching van veelgebruikte data in geheugen (bijv. lyrics, project notes) bij editor open

## Git Conventies

- **Branch model**: `main` (production) / `develop` (integratie) / `feature/*` / `fix/*`
- **Commit messages**: Conventional Commits (`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`)

## Development Commands

```bash
cargo build                  # Debug build
cargo build --release        # Release build
cargo test                   # Unit + integration tests
cargo clippy                 # Lint
cargo fmt                    # Format

# Plugin bundeling (na build)
# macOS AU: copy .component naar ~/Library/Audio/Plug-Ins/Components/
# macOS VST3: copy .vst3 naar ~/Library/Audio/Plug-Ins/VST3/
# Windows VST3: copy .vst3 naar C:\Program Files\Common Files\VST3\
```

## nih-plug Specifieke Conventies

- **Parameters**: Alles wat de gebruiker wil automatiseren moet een `FloatParam`, `IntParam`, etc. zijn. Lyrics, notes, en tekstuele content zijn geen parameters — die gaan in de DB.
- **State serialization**: nih-plug's `Plugin::state()` / `Plugin::load_state()` voor preset opslag. Gebruik dit voor settings die mee moeten met project saves.
- **Editor size**: Startgrootte 900×600. Respecteer DAW resize events.
- **Process context**: Gebruik `ProcessContext::transport()` voor BPM/tempo uit de DAW (essentieel — lees DAW BPM, niet alleen audio detectie).

## Wat te vermijden

❌ React, webview, of enige web-technologie in de plugin  
❌ `std::sync::Mutex` in audio callback  
❌ SQLite queries in real-time thread  
❌ `println!` / `eprintln!` in release builds (gebruik `nih_plug::util::nih_debug_assert!` voor debug logging)  
❌ Inline styling — gebruik altijd `ui/theme.rs` constants  
❌ Emoji in UI (gebruik Unicode symbolen of custom icons)

## Wat te doen

✅ egui's `Frame` en `Stroke` voor consistente glas-effecten  
✅ Pre-allocatie bij `initialize()`  
✅ `crossbeam` channels voor thread-communicatie  
✅ `Atomic` types voor shared flags tussen threads  
✅ `nih_plug::prelude::*` als standaard import  
✅ `#[derive(Params)]` macro correct gebruiken voor parameters  
✅ Transport BPM uitlezen uit DAW als primaire BPM bron (audio detectie = fallback)
