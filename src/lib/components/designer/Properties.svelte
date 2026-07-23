<script lang="ts">
  import type { FormDef, TagDefinition, WidgetDef } from "$lib/types";
  import {
    updateWidget,
    updateFormMeta,
    selectedWidgetId,
    reorderWidget,
    groupSelectedWidgets,
    ungroupSelectedWidgets,
    addNewForm,
    deleteForm,
  } from "$lib/stores/app";

  interface Props {
    widget: WidgetDef | null;
    form: FormDef | null;
    tags: TagDefinition[];
  }

  let { widget, form, tags }: Props = $props();

  function setCfg(key: string, value: unknown) {
    if (!widget) return;
    updateWidget({
      id: widget.id,
      config: { ...widget.config, [key]: value },
    });
  }

  function setField<K extends keyof WidgetDef>(key: K, value: WidgetDef[K]) {
    if (!widget) return;
    updateWidget({ id: widget.id, [key]: value });
  }

  function parseColorAlpha(val: string): { hex6: string; alpha: number } {
    if (!val || typeof val !== "string") return { hex6: "#ffffff", alpha: 100 };
    const str = val.trim().toLowerCase();
    if (str === "transparent") return { hex6: "#ffffff", alpha: 0 };

    if (str.startsWith("#") && str.length === 9) {
      const hex6 = str.slice(0, 7);
      const aHex = str.slice(7, 9);
      const alpha = Math.round((parseInt(aHex, 16) / 255) * 100);
      return { hex6, alpha: isNaN(alpha) ? 100 : alpha };
    }

    if (str.startsWith("#") && str.length === 5) {
      const r = str[1], g = str[2], b = str[3], a = str[4];
      const hex6 = `#${r}${r}${g}${g}${b}${b}`;
      const alpha = Math.round((parseInt(`${a}${a}`, 16) / 255) * 100);
      return { hex6, alpha: isNaN(alpha) ? 100 : alpha };
    }

    if (str.startsWith("rgba")) {
      const m = str.match(/rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*([\d.]+))?\)/);
      if (m) {
        const r = Math.min(255, parseInt(m[1])).toString(16).padStart(2, "0");
        const g = Math.min(255, parseInt(m[2])).toString(16).padStart(2, "0");
        const b = Math.min(255, parseInt(m[3])).toString(16).padStart(2, "0");
        const a = m[4] !== undefined ? Math.round(parseFloat(m[4]) * 100) : 100;
        return { hex6: `#${r}${g}${b}`, alpha: isNaN(a) ? 100 : a };
      }
    }

    if (str.startsWith("#") && str.length === 7) {
      return { hex6: str, alpha: 100 };
    }

    if (str.startsWith("#") && str.length === 4) {
      const r = str[1], g = str[2], b = str[3];
      return { hex6: `#${r}${r}${g}${g}${b}${b}`, alpha: 100 };
    }

    return { hex6: "#ffffff", alpha: 100 };
  }

  function stringifyColorAlpha(hex6: string, alphaPercent: number): string {
    if (alphaPercent <= 0) return "transparent";
    const cleanHex = hex6.startsWith("#") ? hex6.slice(0, 7) : "#ffffff";
    if (alphaPercent >= 100) return cleanHex;
    const aByte = Math.round((alphaPercent / 100) * 255)
      .toString(16)
      .padStart(2, "0");
    return `${cleanHex}${aByte}`;
  }
</script>

<div class="panel" style:height="100%;border:none;border-left:1px solid var(--vs-border)">
  <div class="panel-header">Properties</div>
  <div class="panel-body">
    {#if widget}
      <!-- Z-Order Layer Toolbar & Grouping -->
      <div class="layer-toolbar">
        <span class="layer-title">Layer Order & Grouping</span>
        <div class="layer-btns">
          <button title="Bring to Front" onclick={() => reorderWidget(widget.id, "bring_to_front")}>⇞ Front</button>
          <button title="Bring Forward" onclick={() => reorderWidget(widget.id, "bring_forward")}>↑ Up</button>
          <button title="Send Backward" onclick={() => reorderWidget(widget.id, "send_backward")}>↓ Down</button>
          <button title="Send to Back" onclick={() => reorderWidget(widget.id, "send_to_back")}>⇟ Back</button>
        </div>
        <div class="group-btns">
          <button class="btn-grp" onclick={() => groupSelectedWidgets()}>🔗 Grupuj (Group)</button>
          <button class="btn-ungrp" onclick={() => ungroupSelectedWidgets()}>🔓 Rozgrupuj (Ungroup)</button>
        </div>
      </div>

      <table class="props-table">
        <thead>
          <tr><th colspan="2">Widget · {widget.widget_type}</th></tr>
        </thead>
        <tbody>
          <tr>
            <td>ID</td>
            <td><input value={widget.id} disabled /></td>
          </tr>
          <tr>
            <td>Group ID</td>
            <td>
              <input
                value={widget.group_id ?? "(none)"}
                placeholder="No group"
                onchange={(e) => setField("group_id", e.currentTarget.value || null)}
              />
            </td>
          </tr>
          <tr>
            <td>X</td>
            <td>
              <input
                type="number"
                value={widget.x}
                onchange={(e) => setField("x", Number(e.currentTarget.value))}
              />
            </td>
          </tr>
          <tr>
            <td>Y</td>
            <td>
              <input
                type="number"
                value={widget.y}
                onchange={(e) => setField("y", Number(e.currentTarget.value))}
              />
            </td>
          </tr>
          <tr>
            <td>Width</td>
            <td>
              <input
                type="number"
                value={widget.w}
                onchange={(e) => setField("w", Number(e.currentTarget.value))}
              />
            </td>
          </tr>
          <tr>
            <td>Height</td>
            <td>
              <input
                type="number"
                value={widget.h}
                onchange={(e) => setField("h", Number(e.currentTarget.value))}
              />
            </td>
          </tr>
          <tr>
            <td>Z-Index</td>
            <td>
              <input
                type="number"
                value={widget.z}
                onchange={(e) => setField("z", Number(e.currentTarget.value))}
              />
            </td>
          </tr>
          <tr>
            <td>Tag</td>
            <td>
              <select
                value={widget.tag_id ?? ""}
                onchange={(e) =>
                  setField("tag_id", e.currentTarget.value || null)}
              >
                <option value="">(none)</option>
                {#each tags as t}
                  <option value={t.id}>{t.name}</option>
                {/each}
              </select>
            </td>
          </tr>
        </tbody>
      </table>

      <table class="props-table">
        <thead>
          <tr><th colspan="2">Appearance & Behavior</th></tr>
        </thead>
        <tbody>
          {#each Object.entries(widget.config ?? {}) as [key, val]}
            <tr>
              <td>{key}</td>
              <td>
                {#if typeof val === "boolean"}
                  <select
                    value={val ? "true" : "false"}
                    onchange={(e) => setCfg(key, e.currentTarget.value === "true")}
                  >
                    <option value="true">true</option>
                    <option value="false">false</option>
                  </select>
                {:else if typeof val === "number"}
                  <input
                    type="number"
                    value={val}
                    onchange={(e) => setCfg(key, Number(e.currentTarget.value))}
                  />
                {:else if key.toLowerCase().includes("color")}
                  {@const colorInfo = parseColorAlpha(String(val ?? "#000000"))}
                  <div class="color-picker-box">
                    <div class="color-row">
                      <input
                        type="color"
                        value={colorInfo.hex6}
                        onchange={(e) => {
                          const newColor = stringifyColorAlpha(e.currentTarget.value, colorInfo.alpha);
                          setCfg(key, newColor);
                        }}
                      />
                      <input
                        type="text"
                        value={String(val ?? "")}
                        placeholder="#RRGGBBAA / rgba"
                        onchange={(e) => setCfg(key, e.currentTarget.value)}
                      />
                    </div>
                    <div class="alpha-row">
                      <span class="alpha-label">Opacity: {colorInfo.alpha}%</span>
                      <input
                        type="range"
                        min="0"
                        max="100"
                        step="1"
                        value={colorInfo.alpha}
                        oninput={(e) => {
                          const a = Number(e.currentTarget.value);
                          const newColor = stringifyColorAlpha(colorInfo.hex6, a);
                          setCfg(key, newColor);
                        }}
                      />
                    </div>
                  </div>
                {:else}
                  <input
                    value={String(val ?? "")}
                    onchange={(e) => setCfg(key, e.currentTarget.value)}
                  />
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else if form}
      {@const bgInfo = parseColorAlpha(form.background)}
      <table class="props-table">
        <thead>
          <tr><th colspan="2">Form · {form.name}</th></tr>
        </thead>
        <tbody>
          <tr>
            <td>Name</td>
            <td>
              <input
                value={form.name}
                onchange={(e) => updateFormMeta({ name: e.currentTarget.value })}
              />
            </td>
          </tr>
          <tr>
            <td>Width</td>
            <td>
              <input
                type="number"
                value={form.width}
                onchange={(e) => updateFormMeta({ width: Number(e.currentTarget.value) })}
              />
            </td>
          </tr>
          <tr>
            <td>Height</td>
            <td>
              <input
                type="number"
                value={form.height}
                onchange={(e) => updateFormMeta({ height: Number(e.currentTarget.value) })}
              />
            </td>
          </tr>
          <tr>
            <td>Background</td>
            <td>
              <div class="color-picker-box">
                <div class="color-row">
                  <input
                    type="color"
                    value={bgInfo.hex6}
                    onchange={(e) => {
                      const newColor = stringifyColorAlpha(e.currentTarget.value, bgInfo.alpha);
                      updateFormMeta({ background: newColor });
                    }}
                  />
                  <input
                    type="text"
                    value={form.background}
                    placeholder="#RRGGBBAA / rgba"
                    onchange={(e) => updateFormMeta({ background: e.currentTarget.value })}
                  />
                </div>
                <div class="alpha-row">
                  <span class="alpha-label">Opacity: {bgInfo.alpha}%</span>
                  <input
                    type="range"
                    min="0"
                    max="100"
                    step="1"
                    value={bgInfo.alpha}
                    oninput={(e) => {
                      const a = Number(e.currentTarget.value);
                      const newColor = stringifyColorAlpha(bgInfo.hex6, a);
                      updateFormMeta({ background: newColor });
                    }}
                  />
                </div>
              </div>
            </td>
          </tr>
          <tr>
            <td>Grid Snap</td>
            <td>
              <input
                type="number"
                value={form.grid}
                onchange={(e) => updateFormMeta({ grid: Number(e.currentTarget.value) })}
              />
            </td>
          </tr>
        </tbody>
      </table>

      <div style:padding="10px 8px" style:display="flex" style:flex-direction="column" style:gap="6px">
        <button
          class="btn-grp"
          style:width="100%"
          onclick={() => addNewForm()}
        >
          ➕ Create New Screen
        </button>
        <button
          class="danger"
          style:width="100%"
          onclick={() => deleteForm(form.id)}
        >
          🗑 Delete Current Screen
        </button>
      </div>
    {:else}
      <p style:padding="12px" style:color="var(--vs-text-dim)">No selection</p>
    {/if}

    {#if $selectedWidgetId}
      <div style:padding="8px">
        <button
          class="danger"
          style:width="100%"
          onclick={() => {
            const ev = new KeyboardEvent("keydown", { key: "Delete" });
            window.dispatchEvent(ev);
          }}
        >
          Delete widget
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .layer-toolbar {
    padding: 8px;
    background: #1e1e1e;
    border-bottom: 1px solid #3c3c3c;
  }
  .layer-title {
    font-size: 10px;
    font-weight: 800;
    color: #aaaaaa;
    text-transform: uppercase;
    display: block;
    margin-bottom: 4px;
  }
  .layer-btns {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 4px;
  }
  .layer-btns button {
    background: #333333;
    color: #ffffff;
    border: 1px solid #444444;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 700;
    padding: 4px 2px;
    cursor: pointer;
  }
  .layer-btns button:hover {
    background: #444444;
  }
  .group-btns {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px;
    margin-top: 6px;
  }
  .group-btns button {
    border-radius: 4px;
    font-size: 10px;
    font-weight: 700;
    padding: 4px 6px;
    cursor: pointer;
  }
  .btn-grp {
    background: #1e3a8a;
    color: #ffffff;
    border: 1px solid #3b82f6;
  }
  .btn-grp:hover {
    background: #2563eb;
  }
  .btn-ungrp {
    background: #854d0e;
    color: #ffffff;
    border: 1px solid #eab308;
  }
  .btn-ungrp:hover {
    background: #ca8a04;
  }
  .color-picker-box {
    display: flex;
    flex-direction: column;
    gap: 4px;
    width: 100%;
  }
  .color-row {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .color-row input[type="color"] {
    width: 24px;
    height: 22px;
    padding: 0;
    border: 1px solid #666;
    cursor: pointer;
  }
  .color-row input[type="text"] {
    flex: 1;
  }
  .alpha-row {
    display: flex;
    align-items: center;
    gap: 6px;
    background: #181818;
    padding: 2px 4px;
    border-radius: 3px;
    border: 1px solid #333333;
  }
  .alpha-label {
    font-size: 9px;
    font-weight: 700;
    color: #3b82f6;
    white-space: nowrap;
    min-width: 68px;
  }
  .alpha-row input[type="range"] {
    flex: 1;
    height: 12px;
    accent-color: #3b82f6;
    cursor: pointer;
  }
</style>
