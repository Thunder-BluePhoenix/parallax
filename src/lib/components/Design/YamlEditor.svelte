<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as monaco from "monaco-editor";

  let { value = $bindable(), onChange } = $props<{ value: string; onChange: (val: string) => void }>();
  let editorContainer: HTMLDivElement;
  let editor: monaco.editor.IStandaloneCodeEditor;

  onMount(() => {
    // Basic setup for monaco
    editor = monaco.editor.create(editorContainer, {
      value,
      language: "yaml",
      theme: "vs-dark",
      minimap: { enabled: false },
      automaticLayout: true,
      scrollBeyondLastLine: false,
      fontSize: 12,
      fontFamily: "var(--font-mono)",
      padding: { top: 12, bottom: 12 },
    });

    editor.onDidChangeModelContent(() => {
      const currentVal = editor.getValue();
      value = currentVal;
      onChange(currentVal);
    });
  });

  onDestroy(() => {
    if (editor) editor.dispose();
  });
</script>

<div class="editor-wrap" bind:this={editorContainer}></div>

<style>
  .editor-wrap {
    width: 100%;
    height: 100%;
    min-height: 0;
  }
</style>
