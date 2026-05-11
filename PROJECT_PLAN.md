# BeatPartner VST Plugin — Project Plan

## 1. Project Overzicht

| Specificatie     | Waarde                          |
| ---------------- | ------------------------------- |
| **Naam**         | BeatPartner Plugin              |
| **Type**         | VST3 / AU / CLAP Plugin         |
| **Stack**        | Rust + nih-plug + egui          |
| **Database**     | SQLite (rusqlite)               |
| **UI Framework** | egui (immediate mode GUI)       |
| **License**      | MIT                             |
| **Platforms**    | macOS (primary), Windows, Linux |

## 2. Doel & Visie

Een **standalone VST/AU/CLAP plugin** die binnen elke DAW draait en producers begeleidt met tools, guides, notities en real-time audio analyse. In tegenstelling tot de desktop companion app heeft deze plugin **direct toegang tot de DAW-audio** en kan deze real-time BPM/key detectie uitvoeren op de huidige track of master bus.

**Target**: Beginners tot ervaren producers  
**Design**: Donker thema met glasmorfisme — clean, minimalistisch, non-intrusief  
**Kernprincipe**: Producer-focustool, geen chatbot. Alle features zijn deterministisch of opgeslagen gebruikerscontent.

---

## 3. UI/UX Design

### Plugin Window Layout

```
┌─────────────────────────────────────────────────────────────┐
│  [≡]  BeatPartner          [Tab: Guides] [Lyrics] [Theory] [Analyzer] [Settings]  │
├──────────────┬──────────────────────────────────────────────┤
│              │                                              │
│   SIDEBAR    │           MAIN CONTENT AREA                  │
│              │                                              │
│  [Guides]    │  (Guide / Lyrics Editor /                    │
│  [Lyrics]    │   Theory Helper / Spectrum /                 │
│  [Vocals]    │   Settings)                                  │
│  [Theory]    │                                              │
│  [Analyzer]  │                                              │
│              │                                              │
├──────────────┴──────────────────────────────────────────────┤
│  🎵 DAW: 128 BPM | C minor | Phase: Arrangement | 🎤 Ready   │
└─────────────────────────────────────────────────────────────┘
```

### Design Systeem — Donker Glasmorfisme

**Kleuren:**

- **Surface (achtergronden)**: `#020204`, `#08080c`, `#0f0f14`
- **Accenten**: Cyan `#22d3ee`, Purple `#a78bfa`, Magenta `#f472b6`
- **Glas**: `rgba(255,255,255,0.06-0.12)` met 1px rand `rgba(255,255,255,0.10)`
- **Tekst**: Primary `#e0e0e0`, Secondary `#999999`

**Typography:**

- **Primary**: Inter (embedded font)
- **Monospace**: JetBrains Mono (BPM, key, timestamps)
- **Labels**: 11px, secondary text
- **Body**: 13px, primary text
- **Headings**: 15px, medium weight
- **Titles**: 18px, semibold

**Containers & Panels:**

- `Frame::none().fill(GLASS_BG).rounding(12.0).stroke(1.0, BORDER)`
- Inner padding: 12-16px
- Gap tussen panels: 12px

**Interactieve Elementen:**

- Buttons: `SURFACE_SECONDARY` fill, 8px rounding, 1px border
- Hover: iets lichter fill + border
- Active/selected: accent-cyan border

**Transitions & Animations:**

- egui gebruikt geen CSS — animaties via `ui.ctx().animate_bool()` of custom frame-based easing
- Standaard hover: 150ms easing

### Modals & Dialogs

In egui: gebruik `Window` met `frame(...)` styling. Geen `backdrop-filter` beschikbaar — gebruik semi-transparante dark overlay (`Color32::from_rgba_premultiplied(0,0,0,180)`) met solid modal panel.

---

## 4. State Management

### Plugin Parameters (nih-plug Params)

Alleen settings die de DAW moet kunnen automatiseren/opslaan als preset:

```rust
#[derive(Params)]
struct BeatPartnerParams {
    #[id = "analysis_mode"]
    pub analysis_mode: EnumParam<AnalysisMode>,  // Off / BPM / Key / Both

    #[id = "spectrum_smoothing"]
    pub spectrum_smoothing: FloatParam,  // 0.0 - 1.0

    #[id = "bpm_source"]
    pub bpm_source: EnumParam<BpmSource>,  // Auto / DAW Transport

    #[id = "show_spectrum"]
    pub show_spectrum: BoolParam,
}

enum AnalysisMode { Off, Bpm, Key, Both }
enum BpmSource { AutoDetect, DawTransport }
```

### In-Memory State (Editor)

Niet-persistente UI state (bij editor open/close verloren — dat is OK voor een plugin):

```rust
struct EditorState {
    active_tab: Tab,
    lyrics_content: String,
    selected_annotation: Option<AnnotationId>,
    analyzer_visible: bool,
    sidebar_collapsed: bool,
}
```

### Database State (Persistent)

Alles wat bij project herstart moet blijven bestaan:

- Lyrics + annotaties
- Vocal production notes
- Reference track metadata
- Settings/preferences
- Audio analysis cache

---

## 5. Database Schema (SQLite)

```sql
-- Enable foreign key support
PRAGMA foreign_keys = ON;

-- App Preferences (key-value store)
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Projects (elke plugin instantie = één project in de DAW)
CREATE TABLE projects (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL DEFAULT 'Untitled',
  bpm INTEGER,
  key TEXT,
  genre TEXT,
  phase TEXT DEFAULT 'idea',
  notes TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_projects_updated ON projects(updated_at);

-- Progress Tracking
CREATE TABLE progress (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id INTEGER NOT NULL,
  phase TEXT NOT NULL,
  completed BOOLEAN DEFAULT FALSE,
  notes TEXT,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  UNIQUE(project_id, phase)
);

CREATE INDEX idx_progress_project ON progress(project_id);

-- Samples Library (reference vocals)
CREATE TABLE samples (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  path TEXT NOT NULL UNIQUE,
  category TEXT,
  bpm INTEGER,
  key TEXT,
  duration REAL,
  imported_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_samples_category ON samples(category);

-- Presets (vocal chain presets per genre)
CREATE TABLE presets (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  category TEXT,
  settings_json TEXT NOT NULL,
  imported_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_presets_category ON presets(category);

-- Audio Analysis Cache
CREATE TABLE audio_analysis (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  file_path TEXT,
  file_hash TEXT,
  analysis_type TEXT NOT NULL, -- 'bpm', 'key', 'spectrum'
  results_json TEXT NOT NULL,
  analyzed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(file_path, analysis_type)
);

CREATE INDEX idx_audio_analysis_path ON audio_analysis(file_path);
CREATE INDEX idx_audio_hash ON audio_analysis(file_hash);

-- Lyrics (per project)
CREATE TABLE lyrics (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id INTEGER NOT NULL UNIQUE,
  content TEXT NOT NULL DEFAULT '',
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Lyric Annotations
CREATE TABLE lyric_annotations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  lyrics_id INTEGER NOT NULL,
  start_index INTEGER NOT NULL,
  end_index INTEGER NOT NULL,
  tag TEXT NOT NULL CHECK(tag IN ('melody','ad-lib','harmony','flow','emphasis','note')),
  color TEXT,
  note TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (lyrics_id) REFERENCES lyrics(id) ON DELETE CASCADE
);

CREATE INDEX idx_lyric_annotations_lyrics ON lyric_annotations(lyrics_id);

-- Vocal Production Notes (per project)
CREATE TABLE vocal_production_notes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id INTEGER NOT NULL,
  mic_choice TEXT,
  vocal_chain_json TEXT,
  recording_notes TEXT,
  editing_notes TEXT,
  tuning_notes TEXT,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);
```

---

## 6. Error Handling

### Rust Error Types (`src/error.rs`)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BeatPartnerError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Audio analysis error: {0}")]
    AudioAnalysis(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Real-time safety violation: {0}")]
    RealtimeViolation(String),
}

pub type Result<T> = std::result::Result<T, BeatPartnerError>;
```

### UI Error Handling

- **Non-blocking toasts** in egui (niet-modal, auto-dismiss na 3s)
- **Errors loggen** naar file (via `log` + `fern` — NIET in audio thread)
- **Graceful degradation**: analyse faalt → toon "Analysis unavailable", laat rest van UI werken

---

## 7. Audio Architectuur

### Real-Time Audio Thread

```
DAW Audio Buffer → ring_buffer.write() → [NIETS MEER]
                                         ↓
Background Thread ← ring_buffer.read() ←┘
   ↓
FFT / BPM detectie / Key detectie
   ↓
Resultaten → SQLite cache
         → UI update via channel
```

### Lock-Free Ring Buffer

```rust
// src/audio/ring_buffer.rs
pub struct LockFreeRingBuffer {
    buffer: Vec<AtomicF32>,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
    capacity: usize,
}

impl LockFreeRingBuffer {
    pub fn write(&self, samples: &[f32]);
    pub fn read(&self, out: &mut [f32]) -> usize;
}
```

### BPM Detectie (real-time)

- **Methode**: Onset detection + autocorrelation op spectrum
- **Input**: Mono mix van stereo buffer
- **Output**: BPM waarde + confidence score
- **Update rate**: Elke ~1 seconde (niet elke buffer — te duur)
- **Fallback**: Lees BPM uit DAW transport indien beschikbaar

### Key Detectie (real-time)

- **Methode**: Chromagram + Krumhansl-Schmuckler profiel matching
- **Input**: FFT van audio buffer
- **Output**: Key + confidence
- **Update rate**: Elke ~2 seconden

### Spectrum Analyzer

- **Methode**: Real-time FFT visualisatie
- **Smoothing**: Exponentiële moving average (parameter-gestuurd)
- **Output**: 128 frequency bins → egui bar chart

### DAW Transport Sync

```rust
fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, context: &mut impl ProcessContext<Self>) -> ProcessStatus {
    // Lees BPM uit DAW transport (primair)
    if let Some(transport) = context.transport() {
        if let Some(tempo) = transport.tempo {
            self.daw_bpm.store(tempo as u32, Ordering::Relaxed);
        }
        self.is_playing.store(transport.playing, Ordering::Relaxed);
    }

    // Schrijf audio naar ring buffer voor analyse
    self.audio_ring_buffer.write(buffer.as_slice());

    ProcessStatus::Normal
}
```

---

## 8. Feature Roadmap (Gefaseerd)

### Fase 1: Scaffolding & Basis Layout _(must-have)_

- [ ] nih-plug project scaffolding + egui editor
- [ ] Lock-free ring buffer implementatie
- [ ] SQLite database setup + migratie-systeem
- [ ] Error handling framework
- [ ] Basis layout: sidebar + main content area
- [ ] Donker thema + glasmorfisme styling in egui

### Fase 2: Real-Time Audio _(must-have)_

- [ ] Audio ring buffer integratie in `process()`
- [ ] Background thread voor analyse
- [ ] Spectrum analyzer (FFT visualisatie)
- [ ] DAW transport sync (BPM, play state)
- [ ] Audio analysis cache in SQLite
- [ ] Crossbeam channel setup (audio ↔ UI ↔ background)

### Fase 3: Core Tools _(must-have)_

- [ ] **Theory Helper**: chord grids, scales viewer, circle of fifths, guitar diagrams, progression suggestions
- [ ] **Audio Analyzer**: spectrum FFT visualization + peak/RMS meters
- [ ] **Reference Track Manager**: import + metadata (BPM/key/duration)
- [ ] **BPM/Key Detector**: real-time detectie + DAW fallback
- [ ] **Project Manager**: aanmaken, openen, verwijderen van projecten

### Fase 4: Lyrics & Vocals _(must-have)_

- [ ] **Lyrics Editor**: per-project tekstveld met auto-save
- [ ] **Annotatie systeem**: selecteer tekst → kies tag (`melody`, `ad-lib`, `harmony`, `flow`, `emphasis`, `note`)
- [ ] **Highlight rendering** in editor (kleurgecodeerde achtergronden)
- [ ] **Recording Checklist**: stap-voor-stap checklist voor vocal recording
- [ ] **Mic & Chain Advisor**: suggesties voor microfoonkeuze en vocal chain per genre/vocalist
- [ ] **Vocal Production Notes**: per project notities voor recording, editing, tuning
- [ ] **Reference Vocal Library**: import + analyse/metadata

### Fase 5: Vocal Guides _(nice-to-have)_

- [ ] **Comping Guide**: beste practices voor vocal comping
- [ ] **Tuning & Timing Guide**: when/to what extent te tunen/timen
- [ ] **Effect Presets**: vocal chain presets per genre (pop, hip-hop, R&B, rock)

### Fase 6: Polish & Advanced _(nice-to-have)_

- [ ] **Progress tracking dashboard**
- [ ] **Genre-specific guides**
- [ ] **Export** (tekst/Markdown project rapport)
- [ ] **Plugin detection** (scan installed VSTs in DAW — beperkt door SDK)
- [ ] **Resizeable UI** met responsive layout
- [ ] **Preset browser** met zoeken + filteren

---

## 9. Technische Architectuur

```
src/
├── lib.rs                   # Plugin entry (nih-plug wrapper)
├── params.rs                # Plugin parameters
├── editor.rs                # egui editor setup
├── audio/
│   ├── mod.rs
│   ├── processor.rs         # Strict real-time audio callback
│   ├── ring_buffer.rs       # Lock-free SPSC ring buffer
│   ├── detector.rs          # BPM/Key detection algorithms
│   └── spectrum.rs          # FFT spectrum calculation
├── ui/
│   ├── mod.rs
│   ├── app.rs               # Hoofd UI + tab router
│   ├── theme.rs             # Kleuren, fonts, styling constants
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── glass_panel.rs   # Herbruikbaar glas panel
│   │   ├── glass_button.rs  # Styled button
│   │   └── spectrum_view.rs # Spectrum visualizer widget
│   ├── lyrics/
│   │   ├── mod.rs
│   │   ├── editor.rs        # Lyrics text editor
│   │   ├── annotation_toolbar.rs
│   │   └── highlighted_text.rs
│   ├── vocals/
│   │   ├── mod.rs
│   │   ├── checklist.rs
│   │   ├── chain_advisor.rs
│   │   └── notes_editor.rs
│   ├── theory/
│   │   ├── mod.rs
│   │   ├── chord_grid.rs
│   │   ├── scale_viewer.rs
│   │   ├── circle_of_fifths.rs
│   │   └── progression_suggestions.rs
│   ├── analyzer/
│   │   ├── mod.rs
│   │   └── spectrum_panel.rs
│   └── settings.rs          # Plugin settings panel
├── services/
│   ├── mod.rs
│   ├── db_service.rs        # SQLite database (background thread only)
│   ├── analysis_service.rs  # Audio analyse orchestratie
│   └── project_service.rs   # Project CRUD
├── models/
│   ├── mod.rs
│   ├── project.rs
│   ├── lyrics.rs
│   ├── vocal.rs
│   └── analysis.rs
├── constants/
│   └── mod.rs               # Genre data, chord data, vocal presets
└── error.rs                 # BeatPartnerError enum
```

---

## 10. Dependencies (Key)

### Plugin (Rust)

- `nih_plug` — VST3/AU/CLAP wrapper + egui integratie
- `nih_plug_egui` — egui renderer voor nih-plug
- `egui` — Immediate mode GUI
- `rusqlite` — SQLite (background thread)
- `serde` + `serde_json` — Serialization
- `thiserror` — Error handling
- `symphonia` — Audio decoding (voor reference track import)
- `realfft` — FFT voor spectrum + analyse
- `crossbeam` — Lock-free channels
- `log` + `fern` — Logging
- `rfd` — File dialogs (voor import)

---

## 11. Performance Considerations

- **Audio Thread**: < 1ms CPU budget per buffer. Alleen ring buffer write + parameter lezen.
- **Background Thread**: FFT en BPM detectie draaien hier. Target < 50% CPU van één core.
- **Spectrum**: Update UI op 30fps, niet per audio buffer (te duur voor egui).
- **Database**: WAL mode, prepared statements, bulk inserts voor cache.
- **Memory**: Pre-allocatie bij `initialize()`. Geen allocatie in `process()`.
- **egui**: Geen complexe layouts per frame. Cache layout calculations waar mogelijk.

---

## 12. Distributie & Installatie

### Build Targets

```bash
# macOS Universal (ARM64 + x86_64)
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Windows x64
cargo build --release --target x86_64-pc-windows-msvc

# Linux x64
cargo build --release --target x86_64-unknown-linux-gnu
```

### Installatiepaden

- **macOS AU**: `~/Library/Audio/Plug-Ins/Components/BeatPartner.component`
- **macOS VST3**: `~/Library/Audio/Plug-Ins/VST3/BeatPartner.vst3`
- **Windows VST3**: `%PROGRAMFILES%\Common Files\VST3\BeatPartner.vst3`
- **Linux VST3**: `~/.vst3/BeatPartner.vst3`

### CI/CD

- GitHub Actions: build op macOS, Windows, Linux
- Artefacten: `.component`, `.vst3`, `.clap` bundles
- Release: GitHub Releases met binaries + installatie-instructies

---

## 13. Testing Strategie

| Laag                 | Tool         | Scope                                      |
| -------------------- | ------------ | ------------------------------------------ |
| **Rust unit tests**  | `cargo test` | Services, models, detection algorithms     |
| **Rust integration** | `cargo test` | Database operaties, file I/O               |
| **Audio tests**      | Custom       | Sine wave input → verwachte BPM/key output |
| **UI tests**         | Handmatig    | egui rendering op alle platforms           |
| **DAW tests**        | Handmatig    | Logic Pro, Ableton Live, FL Studio, Reaper |

**Real-time safety checks:**

- `clippy` lint voor `std::sync::Mutex` in audio code
- Custom lint: geen `println!` in `src/audio/`
- CI check: `cargo build --release` moet slagen zonder warnings

---

## 14. Database Migraties

- **Tool**: `rusqlite_migration` crate
- **Strategie**: Sequentiële, genummerde SQL migratie-bestanden
- **Locatie**: `migrations/`
- **Uitvoering**: Automatisch bij plugin instantiatie (bij editor open)
- **Rollback**: Niet ondersteund — nieuwe migratie schrijven om te corrigeren

```
migrations/
├── 001_initial_schema.sql
├── 002_add_audio_analysis.sql
└── ...
```

---

## 15. Development Workflow

### Commands

```bash
# Setup
cargo build

# Development (geen "dev server" — compileer en laad in DAW)
cargo build --release
# Kopieer .component/.vst3 naar plugin map, refresh DAW

# Testing
cargo test
cargo clippy
cargo fmt

# Release build
cargo build --release
```

### Git Workflow

- `main`: Production-ready
- `develop`: Integratie branch
- `feature/*`: Feature branches
- `fix/*`: Bug fix branches

### Code Quality

- **clippy**: Strict linting, real-time safety checks
- **rustfmt**: Consistent formatting
- **Conventional Commits**: Structured commit messages

---

## 16. Belangrijke Randgevallen

| Scenario                                 | Handling                                               |
| ---------------------------------------- | ------------------------------------------------------ |
| Plugin op meerdere tracks geïnstantieerd | Elke instantie heeft eigen SQLite DB + state           |
| DAW BPM verandert tijdens playback       | Update direct uit transport, toon in UI                |
| Geen audio input (midi track)            | Toon "No audio input" in analyzer, BPM uit DAW         |
| Plugin window gesloten                   | Audio callback blijft draaien, analyse pauzeert        |
| DAW project opslaan                      | Plugin state (params) mee opgeslagen. DB blijft apart. |
| Sample rate verandert                    | Re-init analyse buffers, cache ongeldig                |
| Buffer size verandert                    | Geen probleem — ring buffer is sample-based            |
