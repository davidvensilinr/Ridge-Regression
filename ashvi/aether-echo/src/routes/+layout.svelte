<script lang="ts">
  import '../app.css';
  let { children } = $props();
</script>

<svelte:head>
  <title>AetherEcho</title>
  <meta name="description" content="AetherEcho — Stealthy AI Interview Assistant" />
</svelte:head>

{@render children()}
