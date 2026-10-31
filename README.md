# UptimeWidget

**Desktop & menu bar notifications for UptimeRobot**

UptimeWidget keeps your website outages in sight. As soon as one of your [UptimeRobot](https://uptimerobot.com) monitors goes down, a small red, always-on-top alert appears on your desktop. It stays there until the site is back up, so an outage can't slip past you the way an e-mail or push notification can.

Runs on **macOS** and **Windows**.

## Features

- **Always-on-top alert:** a compact red tile (80 px high by default) that you can drag anywhere and resize.
  - With a single outage it shows the site, the reason and the duration directly, e.g. `cms.example.com · 502 Bad Gateway · for 2h 14m`.
  - With several outages it shows `3 sites down`. Click it to open a dropdown with every affected site; clicking a site opens it in the UptimeRobot dashboard.
- **Menu bar / tray icon:** a colored dot (red = down, yellow = possibly down, green = all up, grey = not connected or error). On macOS the number of down sites appears next to it. The menu offers:
  - the down sites
  - *Open UptimeRobot dashboard*
  - *Check now*
  - *Mute* (15 min / 30 min / 1 h)
  - *Settings*
- **Hide, but never forget:** hide the alert for 10 minutes or 5 hours. If *another* site goes down in the meantime, it reappears immediately.
- **Guided setup:** connect your account in under a minute. The app opens the right dashboard page and picks up the copied key from your clipboard automatically.
- **Read-only and secure:** the app only needs UptimeRobot's *read-only* API key. The key is stored in the system keychain (macOS Keychain / Windows Credential Manager).
- **Languages:** English, Deutsch, Español, Français, 中文（简体）. It follows the system language by default and can be changed in the settings.
- **Starts at login** by default (can be turned off in the settings).

## Getting started

1. Install and launch UptimeWidget.
2. Click **Open API page** in the setup window. It opens *Integrations & API* in your UptimeRobot dashboard.
3. In the **API** section, click the copy icon next to **Read-only API key**. If you don't have one yet, create it with **Create**.
4. Switch back to UptimeWidget. The key is detected and verified automatically.

That's it. UptimeWidget checks your monitors every 5 minutes; you can change the interval in the settings.

> The UptimeRobot API is available on all plans, including the free plan (10 requests per minute). One check needs one request per 200 monitors, so even a 1-minute interval stays far below the limit.

## Settings

| Setting | Default | Description |
| --- | --- | --- |
| Check interval | 5 minutes | 1–30 minutes |
| Treat "possibly down" as outage | off | UptimeRobot's `LOOKS_DOWN` state is shown in yellow by default, or in red when enabled |
| Widget size | M (280 × 80) | S / M / L, or any size by dragging the widget's bottom-right corner |
| Show widget when all is up | off | Keeps a small green tile visible at all times |
| Language | system language | Falls back to English if the system language isn't supported |
| Start at login | on | |

## Privacy

- UptimeWidget talks only to `api.uptimerobot.com`. There is no server of its own, no analytics and no telemetry.
- The API key never leaves your machine except in requests to UptimeRobot.
- Only status data is read; the app can't change anything in your account, not even with a full-access key.

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.85 or newer (edition 2024)
- [Node.js](https://nodejs.org) 20 or newer
- Platform requirements for Tauri: see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### Commands

```bash
npm install            # install frontend dependencies
npm run tauri dev      # run the app with hot reload
npm run tauri build    # build installers (.dmg / .app, .msi / .exe)

npm run check          # type-check the frontend
cd src-tauri && cargo test   # run backend unit tests
```

In development builds, the app does **not** register itself as a login item, so the debug binary doesn't start at every login. On macOS you may be asked for keychain access again after each rebuild.

### Architecture

UptimeWidget is a [Tauri 2](https://v2.tauri.app) app with a Rust backend and a Svelte 5 + TypeScript frontend.

```
src-tauri/src/
  api.rs        UptimeRobot API v3 client (pagination, typed errors, rate-limit handling)
  state.rs      app state, status levels and the polling loop
  mute.rs       temporary muting; a new outage ends a mute early
  tray.rs       menu bar / tray icon and menu
  ui.rs         window management, widget placement, language switching
  commands.rs   commands invoked by the frontend
  settings.rs   persisted settings (tauri-plugin-store)
  secrets.rs    API key storage in the system keychain
  i18n.rs       menu bar strings

src/
  routes/widget/      always-on-top alert tile with dropdown
  routes/onboarding/  guided setup
  routes/settings/    settings window
  lib/                API wrappers, translations, formatting, shared components
```

The backend owns all state and keeps polling while every window is hidden. Windows receive updates through the `status-changed` event.

## Author

UptimeWidget started as a small idea and became a signed, cross-platform app with auto-updates in five languages. I'm Peter Bagi, a freelance software developer who turns ideas like this into finished products. If you have one you'd like to see built, get in touch: [peterbagi.de](https://www.peterbagi.de/?utm_source=uptimewidget&utm_medium=readme&utm_campaign=freelance)

## License

[MIT](LICENSE) © 2026 Peter Bagi

## Disclaimer

UptimeWidget is an independent project and is not affiliated with, endorsed by or sponsored by UptimeRobot. "UptimeRobot" is a trademark of its respective owner.
