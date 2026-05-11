-- Enable foreign key support
PRAGMA foreign_keys = ON;

-- App Preferences (key-value store)
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Projects (elke plugin instantie = een project in de DAW)
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
  analysis_type TEXT NOT NULL,
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
