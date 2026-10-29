<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import * as api from "$lib/api";
  import { t } from "$lib/i18n";
  import KeyField from "$lib/KeyField.svelte";
  import PageHeader from "$lib/PageHeader.svelte";

  const win = getCurrentWindow();
  const CLOSE_DELAY_MS = 1800;

  let success = $state<string | null>(null);

  function onConnected(monitorsCount: number) {
    success = t.connected(monitorsCount);
    setTimeout(() => win.close(), CLOSE_DELAY_MS);
  }
</script>

<main class="page">
  <PageHeader title={t.setupTitle} intro={t.setupIntro} />

  {#if success}
    <div class="success" role="status">
      <span class="check" aria-hidden="true">✓</span>
      {success}
    </div>
  {:else}
    <ol class="steps">
      <li>
        <p>{t.step1}</p>
        <button class="btn primary" onclick={() => api.openApiKeysPage()}>{t.openApiPage} ↗</button>
      </li>
      <li>
        <p>{t.step2}</p>
      </li>
      <li>
        <p>{t.step3}</p>
        <KeyField autoDetect onSaved={(info) => onConnected(info.monitorsCount)} />
      </li>
    </ol>

    <footer class="page-footer">
      <p>{t.readOnlyHint}</p>
      <p>{t.storedSecurely}</p>
    </footer>
  {/if}
</main>

<style>
  .steps {
    margin: 0;
    padding: 0;
    list-style: none;
    counter-reset: step;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .steps li {
    counter-increment: step;
    position: relative;
    padding-left: 36px;
  }
  .steps li::before {
    content: counter(step);
    position: absolute;
    left: 0;
    top: 0;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--bg-subtle);
    border: 1px solid var(--border);
    display: grid;
    place-items: center;
    font-size: 12px;
    font-weight: 600;
  }
  .steps p {
    margin: 2px 0 8px;
  }

  .success {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 18px;
    border-radius: 10px;
    background: var(--bg-subtle);
    border: 1px solid var(--border);
    font-weight: 600;
  }
  .check {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--success);
    color: #fff;
    display: grid;
    place-items: center;
  }
</style>
