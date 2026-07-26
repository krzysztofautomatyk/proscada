<script lang="ts">
  import type { ProjectNode } from "$lib/types";
  import { updateProjectNode, log } from "$lib/stores/app";
  import { runProjectScript } from "$lib/services/scriptRuntime";
  import { iconFor } from "$lib/utils/projectTree";

  interface Props {
    node: ProjectNode;
    design?: boolean;
  }

  let { node, design = true }: Props = $props();

  let draft = $state("");
  let previewMd = $state(false);

  $effect(() => {
    draft = node.content ?? "";
    previewMd = false;
  });

  function save() {
    updateProjectNode(node.id, { content: draft });
    log(`Saved ${node.name}`, "ok");
  }

  async function runNow() {
    save();
    const fresh = { ...node, content: draft };
    try {
      await runProjectScript(fresh, {
        type: "custom",
        payload: { source: "editor-run" },
      });
      log(`Script ${node.name} finished`, "ok");
    } catch {
      /* logged in runtime */
    }
  }

  /** Minimal markdown → HTML (headings, bold, lists, code, paragraphs). */
  function renderMd(src: string): string {
    const esc = (s: string) =>
      s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    const lines = src.split("\n");
    const out: string[] = [];
    let inList = false;
    let inCode = false;
    let codeBuf: string[] = [];
    const flushList = () => {
      if (inList) {
        out.push("</ul>");
        inList = false;
      }
    };
    for (const line of lines) {
      if (line.startsWith("```")) {
        if (inCode) {
          out.push(`<pre><code>${esc(codeBuf.join("\n"))}</code></pre>`);
          codeBuf = [];
          inCode = false;
        } else {
          flushList();
          inCode = true;
        }
        continue;
      }
      if (inCode) {
        codeBuf.push(line);
        continue;
      }
      if (/^#{1,3}\s/.test(line)) {
        flushList();
        const level = line.match(/^#+/)![0].length;
        out.push(`<h${level}>${inline(esc(line.replace(/^#+\s*/, "")))}</h${level}>`);
      } else if (/^[-*]\s/.test(line)) {
        if (!inList) {
          out.push("<ul>");
          inList = true;
        }
        out.push(`<li>${inline(esc(line.replace(/^[-*]\s*/, "")))}</li>`);
      } else if (!line.trim()) {
        flushList();
        out.push("<br/>");
      } else {
        flushList();
        out.push(`<p>${inline(esc(line))}</p>`);
      }
    }
    flushList();
    if (inCode) out.push(`<pre><code>${esc(codeBuf.join("\n"))}</code></pre>`);
    return out.join("\n");

    function inline(s: string) {
      return s
        .replace(/`([^`]+)`/g, "<code>$1</code>")
        .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
        .replace(/\*([^*]+)\*/g, "<em>$1</em>");
    }
  }
</script>

<div class="doc-editor">
  <div class="doc-toolbar">
    <span class="title">{iconFor(node.kind)} {node.name}</span>
    <span class="kind">{node.kind}{node.language ? ` · ${node.language}` : ""}</span>
    <span class="spacer"></span>
    {#if node.kind === "markdown"}
      <button type="button" class:primary={previewMd} onclick={() => (previewMd = !previewMd)}>
        {previewMd ? "Edit" : "Preview"}
      </button>
    {/if}
    {#if node.kind === "script"}
      <button type="button" class="run" onclick={runNow} disabled={!design && false}>▶ Run</button>
    {/if}
    {#if design}
      <button type="button" class="primary" onclick={save}>Save</button>
    {/if}
  </div>

  {#if node.kind === "script"}
    <div class="hint">
      Bind from widget Properties → <strong>On Click Script</strong>. API:
      <code>writeTag</code>, <code>getTag</code>, <code>getTagValue</code>, <code>log</code>,
      <code>navigate</code>, <code>ackAlarm</code>. Entry: <code>async function onEvent(event)</code>.
    </div>
  {/if}

  {#if node.kind === "markdown" && previewMd}
    <div class="md-preview">{@html renderMd(draft)}</div>
  {:else if node.kind === "image"}
    <div class="image-viewer">
      <div class="image-box">
        {#if draft}
          <img src={draft} alt={node.name} />
        {:else}
          <div class="no-image">No image data</div>
        {/if}
      </div>
      <div class="image-info">
        <div><strong>Name:</strong> {node.name}</div>
        <div><strong>Kind:</strong> Graphic Asset</div>
        {#if draft}
          <div><strong>Data Size:</strong> {Math.round((draft.length * 0.75) / 1024)} KB</div>
        {/if}
      </div>
    </div>
  {:else}
    <textarea
      class="editor"
      class:code={node.kind === "script"}
      readonly={!design}
      spellcheck={node.kind !== "script"}
      bind:value={draft}
      onkeydown={(e) => {
        if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
          e.preventDefault();
          save();
        }
      }}
    ></textarea>
  {/if}
</div>

<style>
  .doc-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background: #1e1e1e;
  }
  .doc-toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--vs-border);
    background: var(--vs-bg-3);
    flex-shrink: 0;
  }
  .title {
    font-weight: 700;
    color: var(--vs-text-bright);
    font-size: 12px;
  }
  .kind {
    font-size: 10px;
    color: var(--vs-text-dim);
    text-transform: uppercase;
  }
  .spacer {
    flex: 1;
  }
  .doc-toolbar button {
    background: var(--vs-bg-4);
    border: 1px solid var(--vs-border);
    color: var(--vs-text);
    font-size: 11px;
    font-weight: 700;
    padding: 3px 10px;
    border-radius: 3px;
    cursor: pointer;
  }
  .doc-toolbar button.primary {
    background: var(--vs-accent-2);
    border-color: var(--vs-accent);
    color: #fff;
  }
  .doc-toolbar button.run {
    background: #0e7a3d;
    border-color: #16a34a;
    color: #fff;
  }
  .hint {
    padding: 6px 10px;
    font-size: 11px;
    color: var(--vs-text-dim);
    border-bottom: 1px solid var(--vs-border);
    background: #252526;
  }
  .hint code {
    color: #ce9178;
    font-size: 10px;
  }
  .editor {
    flex: 1;
    width: 100%;
    border: none;
    resize: none;
    padding: 12px 14px;
    background: #1e1e1e;
    color: #d4d4d4;
    font-family: "Cascadia Code", "Consolas", "SF Mono", ui-monospace, monospace;
    font-size: 13px;
    line-height: 1.45;
    outline: none;
  }
  .editor.code {
    tab-size: 2;
  }
  .md-preview {
    flex: 1;
    overflow: auto;
    padding: 16px 20px;
    color: #d4d4d4;
    font-size: 13px;
    line-height: 1.55;
  }
  .md-preview :global(h1),
  .md-preview :global(h2),
  .md-preview :global(h3) {
    color: #fff;
    margin: 0.6em 0 0.35em;
  }
  .md-preview :global(code) {
    background: #2d2d30;
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 12px;
  }
  .md-preview :global(pre) {
    background: #2d2d30;
    padding: 10px;
    border-radius: 4px;
    overflow: auto;
  }
  .md-preview :global(ul) {
    padding-left: 1.4em;
  }
  .image-viewer {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 24px;
    overflow: auto;
    background: #181818;
  }
  .image-box {
    max-width: 90%;
    max-height: 70vh;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--vs-border);
    border-radius: 6px;
    padding: 12px;
    background: radial-gradient(circle, #2a2a2a 0%, #1a1a1a 100%);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  }
  .image-box img {
    max-width: 100%;
    max-height: 60vh;
    object-fit: contain;
  }
  .no-image {
    color: var(--vs-text-dim);
    font-size: 12px;
    padding: 32px;
  }
  .image-info {
    display: flex;
    gap: 18px;
    font-size: 11px;
    color: var(--vs-text-dim);
    background: #252526;
    padding: 8px 16px;
    border-radius: 4px;
    border: 1px solid var(--vs-border);
  }
</style>
