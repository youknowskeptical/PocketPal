# Pocket Pal

Pocket Pal is a desktop companion you can look after, play with, and chat with. Its optional AI chat uses an API key from a provider you choose.

## Get started

### Just use the app

Download `pocket-pal.exe` from the project's GitHub Releases and open it. No installer is required.

### Build from source

To build or run Pocket Pal from source, install Rust (which includes Cargo), then download or clone the complete repository. Keep these project files together; downloading only `src` is not enough:

- `src\` — application source code
- `Cargo.toml` and `Cargo.lock` — package and dependency definitions
- `build.rs` and `assets\pocket-pal.ico` — Windows executable icon resource
- `README.md` — project and usage instructions

Do not download the generated `target\` folder. Cargo creates it when building.

From the project folder, run:

```powershell
cargo run
```

To build an optimized executable instead:

```powershell
cargo build --release
```

The executable will be at `target\release\pocket-pal.exe`.

## Using Pocket Pal

- Use the main window tabs to manage your companion, collection, room, settings, and chat.
- Choose a companion and look after it with snack and nap activities.
- Play Star Catcher to earn buttons, then use buttons to unlock companions and room decorations.
- Select **FLOAT** to show your companion in a small floating window. Click the companion to reveal its controls; drag it to move it. The play and chat controls are available in FLOAT mode.
- Use the **X** control in FLOAT mode to return to the main app. Minimize to the system tray to keep FLOAT running in the background. Click the tray icon to restore the main app, or use its menu to show the app or quit.
- Choose whether the floating companion stays above other windows or sits behind them in **Settings**.
- The companion can speak occasional care reminders. AI replies can also be spoken on Windows; toggle this in the floating chat panel.
- On Windows, Pocket Pal creates a desktop shortcut automatically the first time it runs. Enable **Run Pocket Pal when I sign in** in **Settings** to start it automatically after you sign in.

## Set up AI chat

Chat requires your own API key from OpenAI, Anthropic, or Google Gemini. The provider may charge for API usage according to its pricing and account settings.

1. Create an API key in the account dashboard of your chosen provider. Pocket Pal does not provide API access or cover usage costs.
2. Open **Chat** in Pocket Pal (or click the chat control in FLOAT).
3. Select the same provider that issued your key.
4. Enter the key in the password field and select **Save key**.
5. Type a message and send it.

The key is stored in your operating system's credential store, not in Pocket Pal's save file. Chat messages are sent to the selected provider to generate replies. Select **Remove** to delete a saved key. If you use more than one provider, save a key separately for each provider.

## Saved data

On Windows, companion progress is saved in:

```text
%APPDATA%\PocketPal\save.json
```

API keys are stored separately in the Windows credential store.
