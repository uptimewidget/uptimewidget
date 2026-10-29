<script lang="ts">
  import { onMount, tick } from "svelte";
  import { currentMonitor, getCurrentWindow } from "@tauri-apps/api/window";
  import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
  import * as api from "$lib/api";
  import { t } from "$lib/i18n";
  import { clockTime, monitorDetail } from "$lib/format";
  import type { Monitor, StatusView } from "$lib/types";

  const win = getCurrentWindow();
  const DRAG_THRESHOLD = 4;
  const PANEL_GAP = 6;
  const PANEL_MAX_HEIGHT = 380;
  const SCREEN_EDGE = 8;

  let status = $state<StatusView | null>(null);
  let now = $state(Date.now());
  let expanded = $state(false);
  let upward = $state(false);
  let tileHeight = $state(window.innerHeight);
  let panelEl = $state<HTMLElement>();
  let busy = false;

  const monitors = $derived<Monitor[]>(status ? [...status.alerts, ...status.warnings] : []);
  const tone = $derived.by(() => {
    switch (status?.level) {
      case "down":
        return "down";
      case "warning":
        return "warning";
      case "ok":
        return "ok";
      default:
        return "error";
    }
  });

  const tile = $derived.by(() => {
    if (!status) return { title: "", subtitle: "" };
    const { alerts, warnings, error } = status;
    switch (status.level) {
      case "down":
        if (alerts.length === 1) {
          return { title: alerts[0].name, subtitle: monitorDetail(alerts[0].reason, alerts[0].downSince, now) };
        }
        return { title: t.sitesOffline(alerts.length), subtitle: alerts.map((m) => m.name).join(", ") };
      case "warning":
        return {
          title: t.possiblyDown,
          subtitle: warnings.length === 1 ? warnings[0].name : warnings.map((m) => m.name).join(", "),
        };
      case "ok":
        return { title: t.allUp, subtitle: lastChecked() };
      default:
        return {
          title: error === "invalidKey" ? t.invalidKey : error === "rateLimited" ? t.rateLimited : t.noConnection,
          subtitle: lastChecked(),
        };
    }
  });

  function lastChecked(): string {
    const time = clockTime(status?.lastCheck ?? null);
    return time ? `${t.checked} ${time}` : "";
  }

  async function expand() {
    if (expanded || busy) return;
    busy = true;
    try {
      expanded = true;
      await win.setResizable(false);
      await tick();
      const panelHeight = Math.min(panelEl?.scrollHeight ?? 0, PANEL_MAX_HEIGHT);
      const total = tileHeight + PANEL_GAP + panelHeight;

      // Open upwards when there is not enough room below the widget.
      const scale = await win.scaleFactor();
      const pos = (await win.outerPosition()).toLogical(scale);
      const monitor = await currentMonitor();
      if (monitor) {
        const screenBottom = (monitor.position.y + monitor.size.height) / monitor.scaleFactor;
        upward = pos.y + total > screenBottom - SCREEN_EDGE;
      }
      await win.setSize(new LogicalSize(window.innerWidth, total));
      if (upward) {
        await win.setPosition(new LogicalPosition(pos.x, pos.y - (total - tileHeight)));
      }
    } finally {
      busy = false;
    }
  }

  async function collapse() {
    if (!expanded || busy) return;
    busy = true;
    try {
      const expandedHeight = window.innerHeight;
      await win.setSize(new LogicalSize(window.innerWidth, tileHeight));
      if (upward) {
        // The tile sits at the bottom while open upwards; keep it in place.
        const scale = await win.scaleFactor();
        const pos = (await win.outerPosition()).toLogical(scale);
        await win.setPosition(new LogicalPosition(pos.x, pos.y + (expandedHeight - tileHeight)));
      }
      upward = false;
      expanded = false;
      await win.setResizable(true);
    } finally {
      busy = false;
    }
  }

  /** Re-fits the open panel after the list of monitors changed. */
  async function refit() {
    if (!expanded || busy || !panelEl) return;
    await tick();
    const panelHeight = Math.min(panelEl.scrollHeight, PANEL_MAX_HEIGHT);
    const total = tileHeight + PANEL_GAP + panelHeight;
    const delta = total - window.innerHeight;
    if (Math.abs(delta) < 1) return;
    await win.setSize(new LogicalSize(window.innerWidth, total));
    if (upward) {
      const scale = await win.scaleFactor();
      const pos = (await win.outerPosition()).toLogical(scale);
      await win.setPosition(new LogicalPosition(pos.x, pos.y - delta));
    }
  }

  function toggle() {
    if (status?.error === "invalidKey") {
      api.openSettings();
      return;
    }
    if (expanded) collapse();
    else expand();
  }

  // A short press toggles the panel, pulling the tile moves the window.
  let pressStart: { x: number; y: number } | null = null;

  function onPointerDown(event: PointerEvent) {
    if (event.button !== 0) return;
    pressStart = { x: event.screenX, y: event.screenY };
  }

  function onPointerMove(event: PointerEvent) {
    if (!pressStart || !(event.buttons & 1)) return;
    if (Math.hypot(event.screenX - pressStart.x, event.screenY - pressStart.y) > DRAG_THRESHOLD) {
      pressStart = null;
      win.startDragging();
    }
  }

  function onPointerUp() {
    if (pressStart) toggle();
    pressStart = null;
  }

  function onKeyDown(event: KeyboardEvent) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      toggle();
    } else if (event.key === "Escape") {
      collapse();
    }
  }

  function startResize(event: PointerEvent) {
    event.stopPropagation();
    win.startResizeDragging("SouthEast");
  }

  async function hideFor(minutes: number) {
    await collapse();
    await api.mute(minutes);
  }

  async function openMonitor(id: number) {
    await api.openMonitor(id);
    collapse();
  }

  $effect(() => {
    // Track list changes while the panel is open.
    void monitors.length;
    void status?.level;
    refit();
  });

  $effect(() => {
    if (status && !status.widgetVisible) collapse();
  });

  onMount(() => {
    let saveTimer: ReturnType<typeof setTimeout> | undefined;
    const clock = setInterval(() => (now = Date.now()), 30_000);
    const subscriptions = [
      api.subscribeStatus((s) => (status = s)),
      win.onResized(async ({ payload }) => {
        if (expanded || busy) return;
        const size = payload.toLogical(await win.scaleFactor());
        tileHeight = size.height;
        clearTimeout(saveTimer);
        saveTimer = setTimeout(() => api.setWidgetSize(size.width, size.height), 400);
      }),
      win.onFocusChanged(({ payload: focused }) => {
        if (!focused) collapse();
      }),
    ];
    return () => {
      clearInterval(clock);
      clearTimeout(saveTimer);
      subscriptions.forEach((p) => p.then((unlisten) => unlisten()));
    };
  });
</script>

<svelte:head>
  <style>
    html,
    body {
      background: transparent !important;
      overflow: hidden;
      user-select: none;
      -webkit-user-select: none;
    }
  </style>
</svelte:head>

{#if status}
  <div class="root" class:upward style="--tile-h: {tileHeight}px">
    <div
      class="tile {tone}"
      class:open={expanded}
      role="button"
      tabindex="0"
      aria-expanded={expanded}
      onpointerdown={onPointerDown}
      onpointermove={onPointerMove}
      onpointerup={onPointerUp}
      onkeydown={onKeyDown}
    >
      <span class="dot" class:pulse={tone === "down"}></span>
      <div class="text">
        <div class="title">{tile.title}</div>
        {#if tile.subtitle}
          <div class="subtitle">{tile.subtitle}</div>
        {/if}
      </div>
      <svg class="chevron" class:up={upward} viewBox="0 0 24 24" aria-hidden="true">
        <path d="M6 9l6 6 6-6" />
      </svg>
      {#if !expanded}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="grip" onpointerdown={startResize}></div>
      {/if}
    </div>

    {#if expanded}
      <div class="panel" bind:this={panelEl}>
        {#if monitors.length > 0}
          <ul class="list">
            {#each monitors as monitor (monitor.id)}
              <li>
                <button class="row" onclick={() => openMonitor(monitor.id)}>
                  <span class="row-dot {monitor.status}"></span>
                  <span class="row-text">
                    <span class="row-name">{monitor.name}</span>
                    <span class="row-detail">{monitorDetail(monitor.reason, monitor.downSince, now)}</span>
                  </span>
                  <span class="row-open" aria-hidden="true">↗</span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}

        <div class="footer">
          {#if monitors.length > 0 || status.level === "error"}
            <div class="footer-line">
              <span class="muted">{t.hideFor}</span>
              <button class="chip" onclick={() => hideFor(10)}>{t.minutes10}</button>
              <button class="chip" onclick={() => hideFor(300)}>{t.hours5}</button>
            </div>
          {/if}
          <div class="footer-line">
            <span class="muted grow">{status.checking ? t.checking : lastChecked()}</span>
            <button class="chip" onclick={() => api.checkNow()} disabled={status.checking}>⟳ {t.checkNow}</button>
            <button class="chip" onclick={() => api.openDashboard()} title={t.dashboard}>↗</button>
            <button class="chip" onclick={() => api.openSettings()} title={t.settings}>⚙</button>
          </div>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .root {
    display: flex;
    flex-direction: column;
    gap: 6px;
    height: 100vh;
  }

  .root.upward {
    flex-direction: column-reverse;
  }

  .tile {
    --pad: calc(var(--tile-h) * 0.2);
    position: relative;
    flex: 0 0 var(--tile-h);
    height: var(--tile-h);
    display: flex;
    align-items: center;
    gap: calc(var(--pad) * 0.8);
    padding: 0 var(--pad);
    border-radius: min(14px, calc(var(--tile-h) * 0.2));
    color: #fff;
    cursor: pointer;
    overflow: hidden;
    outline: none;
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.12);
  }

  .tile.down {
    background: linear-gradient(135deg, #e5484d, var(--red-strong));
  }
  .tile.warning {
    background: linear-gradient(135deg, #f5a524, #c77700);
  }
  .tile.ok {
    background: linear-gradient(135deg, #3cb179, #23784e);
  }
  .tile.error {
    background: linear-gradient(135deg, #7c7f8a, #55575f);
  }
  .tile:focus-visible {
    box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.7);
  }

  .dot {
    flex: none;
    width: calc(var(--tile-h) * 0.16);
    height: calc(var(--tile-h) * 0.16);
    border-radius: 50%;
    background: #fff;
  }
  .dot.pulse {
    animation: pulse 1.6s ease-in-out infinite;
  }
  @keyframes pulse {
    0%,
    100% {
      box-shadow: 0 0 0 0 rgba(255, 255, 255, 0.7);
    }
    50% {
      box-shadow: 0 0 0 calc(var(--tile-h) * 0.07) rgba(255, 255, 255, 0);
    }
  }

  .text {
    flex: 1;
    min-width: 0;
  }
  .title,
  .subtitle {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .title {
    font-weight: 650;
    font-size: max(12px, calc(var(--tile-h) * 0.235));
    line-height: 1.2;
  }
  .subtitle {
    font-size: max(10px, calc(var(--tile-h) * 0.155));
    opacity: 0.88;
    margin-top: 2px;
  }

  .chevron {
    flex: none;
    /* Never smaller than in the L size (110px tile). */
    width: max(20px, calc(var(--tile-h) * 0.2));
    height: max(20px, calc(var(--tile-h) * 0.2));
    fill: none;
    stroke: currentColor;
    stroke-width: 2.5;
    stroke-linecap: round;
    stroke-linejoin: round;
    opacity: 0.85;
    transition: transform 0.15s;
  }
  .chevron.up {
    transform: rotate(180deg);
  }
  .tile.open .chevron {
    transform: rotate(180deg);
  }
  .tile.open .chevron.up {
    transform: none;
  }

  .grip {
    position: absolute;
    right: 0;
    bottom: 0;
    width: 14px;
    height: 14px;
    cursor: nwse-resize;
    opacity: 0;
    background: linear-gradient(135deg, transparent 50%, rgba(255, 255, 255, 0.6) 50%);
    border-bottom-right-radius: inherit;
    transition: opacity 0.15s;
  }
  .tile:hover .grip {
    opacity: 1;
  }

  .panel {
    flex: none;
    max-height: 380px;
    overflow-y: auto;
    background: var(--panel-bg);
    color: var(--panel-text);
    border: 1px solid var(--panel-border);
    border-radius: 12px;
    font-size: 13px;
  }

  .list {
    list-style: none;
    margin: 0;
    padding: 4px;
  }

  .row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: inherit;
    text-align: left;
  }
  .row:hover,
  .row:focus-visible {
    background: var(--panel-hover);
    outline: none;
  }

  .row-dot {
    flex: none;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #e5484d;
  }
  .row-dot.looksDown {
    background: #f5a524;
  }

  .row-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .row-name,
  .row-detail {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .row-name {
    font-weight: 600;
  }
  .row-detail {
    color: var(--panel-muted);
    font-size: 12px;
  }
  .row-open {
    color: var(--panel-muted);
  }

  .footer {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 10px 10px;
    border-top: 1px solid var(--panel-border);
  }
  .list:empty + .footer,
  .panel > .footer:first-child {
    border-top: 0;
  }

  .footer-line {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .muted {
    color: var(--panel-muted);
    font-size: 12px;
  }
  .grow {
    flex: 1;
  }

  .chip {
    border: 1px solid var(--panel-border);
    background: rgba(255, 255, 255, 0.05);
    color: var(--panel-text);
    border-radius: 6px;
    padding: 3px 8px;
    font-size: 12px;
  }
  .chip:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.14);
  }
  .chip:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
