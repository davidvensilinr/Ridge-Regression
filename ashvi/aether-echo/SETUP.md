# AetherEcho — Developer Setup & Run Guide

## Prerequisites (Windows)

Install these **once** before running AetherEcho:

```powershell
# 1. Rust toolchain
winget install --id Rustlang.Rustup -e

# 2. Restart terminal, then install MSVC target
rustup default stable-msvc

# 3. Node.js 20+
winget install --id OpenJS.NodeJS.LTS -e

# 4. LLVM/Clang (needed by whisper-rs FFI)
winget install --id LLVM.LLVM -e

# 5. CMake
winget install --id Kitware.CMake -e

# WebView2 is pre-installed on Windows 10/11 — no action needed
```

After installing Rust/LLVM, **restart your terminal**.

---

## Running in Development Mode

```powershell
cd aether-echo
npm install          # already done
npm run tauri dev    # starts Vite + Tauri + Rust in watch mode
```

The app will compile Rust (~2–5 min first time), then open the AetherEcho window.

---

## Building Production .exe

```powershell
npm run tauri build
# Output: src-tauri\target\release\bundle\msi\AetherEcho_0.1.0_x64_en-US.msi
#    or:  src-tauri\target\release\bundle\nsis\AetherEcho_0.1.0_x64-setup.exe
```

---

## Downloading the Whisper Model

On first launch, go through the setup wizard — it will download the model automatically.

To download manually:
```powershell
$dir = "$env:APPDATA\com.aetherecho.app\models"
New-Item -ItemType Directory -Force $dir
Invoke-WebRequest `
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin" `
  -OutFile "$dir\ggml-tiny.bin"
```

---

## Project Structure

```
aether-echo/
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs           ← Tauri app builder (entry)
│   │   ├── main.rs          ← thin wrapper
│   │   ├── audio.rs         ← WASAPI loopback capture
│   │   ├── vad.rs           ← energy-based VAD + resampling
│   │   ├── transcriber.rs   ← whisper-rs transcription
│   │   ├── groq_client.rs   ← Groq API HTTP client
│   │   ├── pipeline.rs      ← audio → VAD → transcribe → Groq
│   │   ├── overlay.rs       ← overlay window + WDA_EXCLUDEFROMCAPTURE
│   │   ├── tray.rs          ← system tray
│   │   ├── hotkeys.rs       ← global shortcuts
│   │   ├── keyring_store.rs ← Windows Credential Manager
│   │   ├── commands.rs      ← Tauri IPC commands
│   │   └── state.rs         ← shared AppState
│   ├── capabilities/
│   │   ├── main.json        ← main window permissions
│   │   └── overlay.json     ← overlay permissions (minimal)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── routes/
│   │   ├── +page.svelte     ← main app router
│   │   ├── +layout.svelte   ← root layout
│   │   └── overlay/
│   │       └── +page.svelte ← transparent overlay UI
│   ├── lib/
│   │   ├── api.ts           ← Tauri invoke wrappers
│   │   ├── stores/app.ts    ← Svelte 5 rune stores
│   │   └── components/
│   │       ├── SetupWizard.svelte
│   │       ├── MainScreen.svelte
│   │       └── SettingsPanel.svelte
│   └── app.css              ← global design system
└── package.json
```

---

## Security Architecture

| Concern | Solution |
|---------|----------|
| API key | Windows Credential Manager via `keyring` crate |
| Overlay stealth | `WDA_EXCLUDEFROMCAPTURE` via Win32 API |
| Network | Only `api.groq.com` — no telemetry |
| Audio | Stops immediately when not listening |
| Permissions | Minimal Tauri capabilities |

---

## Hotkeys

| Shortcut | Action |
|----------|--------|
| `Ctrl+Alt+Shift+O` | Show / Hide overlay |
| `Ctrl+Alt+Shift+L` | Start / Stop listening |

Both are customizable in Settings → Hotkeys.

---

## Troubleshooting

**"No default render device"** — Make sure speakers/headphones are connected and set as default in Windows Sound settings.

**"Whisper model not found"** — Go to Settings → Model → Download.

**"Invalid API key"** — Get a free key at https://console.groq.com/keys

**Build fails with LLVM error** — Install LLVM: `winget install LLVM.LLVM`, then restart terminal.

**Overlay not working** — Run as administrator once to apply `WDA_EXCLUDEFROMCAPTURE` (Windows 10 20H2+ required for this feature).
