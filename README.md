# Isotope

A browser for containerized web apps, like Franz, Ferdium or Rambox, built with Rust and Tauri instead of Electron.

## Features

- **Sidebar** with one icon per web app, plus unread badges read from page titles
- **Folders** that group apps behind a single icon
- **Profiles**: apps on the same profile share cookies and logins; other profiles stay isolated
- **Hibernation**: idle apps are unloaded after a timeout and restored on click
- **External links** open in your default browser
- Native notifications, drag-and-drop ordering, and per-app user agents

## Development

Requirements: [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/), and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.

```sh
npm install
npm run tauri dev
```

Tests:

```sh
cargo test --workspace
npm test
```

## Layout

- `crates/isotope-core`: platform-independent logic (config, profiles, hibernation, link routing)
- `src-tauri`: the Tauri app that hosts the app webviews
- `src`: the SvelteKit shell UI

## Status

Early development. Tested on Windows only.
