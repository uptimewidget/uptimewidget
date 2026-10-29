<script lang="ts">
  interface Props {
    title: string;
    intro?: string;
  }

  let { title, intro }: Props = $props();
</script>

<header>
  <div class="logo" aria-hidden="true"><span></span></div>
  <h1>{title}</h1>
  {#if intro}
    <p>{intro}</p>
  {/if}
</header>

<style>
  header {
    margin-bottom: 20px;
  }

  .logo {
    width: 40px;
    height: 40px;
    border-radius: 11px;
    background: linear-gradient(135deg, #e5484d, var(--red-strong));
    display: grid;
    place-items: center;
    margin-bottom: 14px;
  }
  .logo span {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #fff;
  }

  h1 {
    font-size: 20px;
    margin: 0 0 6px;
  }

  p {
    margin: 0;
    color: var(--text-muted);
  }
</style>
