<script lang="ts">
  import type { FormDef, TagDefinition, WidgetDef } from "$lib/types";
  import { updateWidget, updateFormMeta, selectedWidgetId } from "$lib/stores/app";

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
</script>

<div class="panel" style:height="100%;border:none;border-left:1px solid var(--vs-border)">
  <div class="panel-header">Properties</div>
  <div class="panel-body">
    {#if widget}
      <table class="props-table">
        <thead>
          <tr><th colspan="2">Widget · {widget.widget_type}</th></tr>
        </thead>
        <tbody>
          <tr>
            <td>Name</td>
            <td><input value={widget.id} disabled /></td>
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
            <td>Z-Order</td>
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
          <tr><th colspan="2">Appearance / Behavior</th></tr>
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
              <input
                value={form.background}
                onchange={(e) => updateFormMeta({ background: e.currentTarget.value })}
              />
            </td>
          </tr>
          <tr>
            <td>Grid</td>
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
      <p style:padding="10px" style:color="var(--vs-text-dim)" style:font-size="11px">
        Select a control on the form designer to edit widget properties (WinForms-style PropertyGrid).
      </p>
    {:else}
      <p style:padding="12px" style:color="var(--vs-text-dim)">No selection</p>
    {/if}

    {#if $selectedWidgetId}
      <div style:padding="8px">
        <button
          class="danger"
          style:width="100%"
          onclick={() => {
            /* delete via keyboard / menu */
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
