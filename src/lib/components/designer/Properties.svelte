<script lang="ts">
  import type { FormDef, TagDefinition, WidgetDef } from "$lib/types";
  import {
    project,
    selectedFormId,
    updateWidget,
    updateFormMeta,
    selectedWidgetId,
    reorderWidget,
    groupSelectedWidgets,
    ungroupSelectedWidgets,
    addNewForm,
    deleteForm,
    scriptNodes,
    importImageFiles,
  } from "$lib/stores/app";
  import { FONT_OPTIONS } from "$lib/utils/dynamics";
  import ConditionEditor from "./ConditionEditor.svelte";
  import { normalizeProjectDesignSystem } from "$lib/utils/designSystem";
  import { collectProjectImages } from "$lib/utils/projectTree";
  import VerticalScrollControls from "./VerticalScrollControls.svelte";

  interface Props {
    widget: WidgetDef | null;
    form: FormDef | null;
    tags: TagDefinition[];
  }

  let { widget, form, tags }: Props = $props();
  let scrollContainer = $state<HTMLDivElement | null>(null);
  let propsFileInputEl = $state<HTMLInputElement | null>(null);
  const designSystem = $derived(normalizeProjectDesignSystem($project?.design_system));

  function triggerPropsImageUpload() {
    propsFileInputEl?.click();
  }

  async function handlePropsImageUpload(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      const ids = await importImageFiles(input.files, null);
      if (ids.length > 0) {
        const tree = $project?.tree ?? [];
        const addedNode = tree.find((n) => n.id === ids[0]);
        if (addedNode?.content) {
          setCfg("src", addedNode.content);
        }
      }
      input.value = "";
    }
  }

  function setCfg(key: string, value: unknown) {
    if (!widget) return;
    // Always merge against live project state (not stale props snapshot)
    updateWidget({
      id: widget.id,
      config: { ...(widget.config ?? {}), [key]: value },
    });
  }

  /** Immediate text edits (input/textarea) — fire on every keystroke. */
  function setCfgLive(key: string, value: unknown) {
    setCfg(key, value);
  }

  function cfgStr(key: string, d = "") {
    return String(widget?.config?.[key] ?? d);
  }
  function cfgNum(key: string, d = 0) {
    return Number(widget?.config?.[key] ?? d);
  }

  function setField<K extends keyof WidgetDef>(key: K, value: WidgetDef[K]) {
    if (!widget) return;
    updateWidget({ id: widget.id, [key]: value });
  }

  /** Keys handled by structured editors (hidden from raw dump). */
  const STRUCTURED_KEYS = new Set([
    "text",
    "title",
    "label",
    "fontFamily",
    "fontSize",
    "fontWeight",
    "fontStyle",
    "textColor",
    "titleColor",
    "titleFontSize",
    "bgColor",
    "borderColor",
    "borderWidth",
    "borderRadius",
    "borderStyle",
    "align",
    "vAlign",
    "onLabel",
    "offLabel",
    "onColor",
    "offColor",
    "trueLabel",
    "falseLabel",
    "trueColor",
    "falseColor",
    "fillColor",
    "trackColor",
    "warnColor",
    "alarmColor",
    "unit",
    "decimals",
    "min",
    "max",
    "warn",
    "alarm",
    "warningAt",
    "alarmAt",
    "step",
    "variant",
    "shadow",
    "blinkMode",
    "blinkTagId",
    "blinkBit",
    "blinkVal",
    "blinkSpeedMs",
    "scrollMode",
    "scrollTagId",
    "scrollTopTagId",
    "scrollBit",
    "scrollVal",
    "scrollSpeedSec",
    "scrollDir",
    "visibilityMode",
    "visibilityTagId",
    "visibilityBit",
    "visibilityVal",
    // line widget
    "x1",
    "y1",
    "x2",
    "y2",
    "stroke",
    "strokeWidth",
    "lineStyle",
    "startCap",
    "endCap",
    "capSize",
    // embedded_screen widget
    "target_form_id",
    "tag_prefix",
    "tag_overrides",
    "scale_mode",
    // image widget
    "src",
    "trueSrc",
    "fit",
    "alt",
    "stateMode",
    "stateTagId",
    "stateBit",
    "stateVal",
    "styleClassId",
    "fontTokenId",
    "animationPresetId",
    "animationMode",
    "animationTagId",
    "animationBit",
    "animationVal",
  ]);

  const rawConfigEntries = $derived(
    Object.entries(widget?.config ?? {}).filter(([k]) => !STRUCTURED_KEYS.has(k)),
  );

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
  <div class="panel-header">
    <span>Properties</span>
    <VerticalScrollControls target={scrollContainer} />
  </div>
  <div class="panel-body scrollable-panel-body" bind:this={scrollContainer}>
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
            <td>Tag (Variable)</td>
            <td>
              <select
                value={widget.tag_id ?? ""}
                onchange={(e) =>
                  setField("tag_id", e.currentTarget.value || null)}
              >
                <option value="">(none - static)</option>
                {#each tags as t}
                  <option value={t.id}>
                    {t.name} ({t.id} · {t.data_type}{t.unit ? ` · ${t.unit}` : ""})
                  </option>
                {/each}
              </select>
            </td>
          </tr>
          <tr>
            <td>🔒 Lock</td>
            <td>
              <select
                value={widget.locked ? "true" : "false"}
                onchange={(e) => setField("locked", e.currentTarget.value === "true")}
              >
                <option value="false">Unlocked (move/resize)</option>
                <option value="true">Locked (no move/resize)</option>
              </select>
            </td>
          </tr>
          <tr>
            <td colspan="2" class="section">Project Design System</td>
          </tr>
          <tr>
            <td>Style class</td>
            <td>
              <select value={cfgStr("styleClassId", "style-default")} onchange={(e) => setCfg("styleClassId", e.currentTarget.value)}>
                {#each designSystem.styles as style}
                  <option value={style.id}>{style.name}</option>
                {/each}
              </select>
            </td>
          </tr>
          <tr>
            <td>Font token</td>
            <td>
              <select value={cfgStr("fontTokenId", "none")} onchange={(e) => setCfg("fontTokenId", e.currentTarget.value)}>
                <option value="none">(none / custom settings)</option>
                {#each designSystem.fonts as font}
                  <option value={font.id}>{font.name}</option>
                {/each}
              </select>
            </td>
          </tr>
          <tr>
            <td>Animation preset</td>
            <td>
              <select value={cfgStr("animationPresetId", "anim-none")} onchange={(e) => setCfg("animationPresetId", e.currentTarget.value)}>
                {#each designSystem.animations as animation}
                  <option value={animation.id}>{animation.name}</option>
                {/each}
              </select>
            </td>
          </tr>
          <tr>
            <td colspan="2" class="section">Events / Scripts</td>
          </tr>
          <tr>
            <td>On Click Script</td>
            <td>
              <select
                value={cfgStr("onClickScriptId", "")}
                onchange={(e) => setCfg("onClickScriptId", e.currentTarget.value)}
              >
                <option value="">(none)</option>
                {#each $scriptNodes as s}
                  <option value={s.id}>{s.name}</option>
                {/each}
              </select>
            </td>
          </tr>
          {#if widget.widget_type === "write_button"}
            <tr>
              <td>Also write after script</td>
              <td>
                <select
                  value={cfgStr("alsoWriteAfterScript", "false")}
                  onchange={(e) =>
                    setCfg("alsoWriteAfterScript", e.currentTarget.value === "true")}
                >
                  <option value="false">No (script only)</option>
                  <option value="true">Yes</option>
                </select>
              </td>
            </tr>
          {/if}
        </tbody>
      </table>

      <!-- Analog Value & Range Settings for numeric, meter, bar, tank, numeric_input, label -->
      {#if ["numeric", "meter", "bar", "tank", "numeric_input", "label"].includes(widget.widget_type)}
        {@const boundTagDef = widget.tag_id ? tags.find((t) => t.id === widget.tag_id) : undefined}
        <table class="props-table">
          <thead>
            <tr><th colspan="2">Analog Value & Range Settings</th></tr>
          </thead>
          <tbody>
            <tr>
              <td>Unit</td>
              <td>
                <input
                  type="text"
                  placeholder={boundTagDef?.unit ? `Default: ${boundTagDef.unit}` : "e.g. °C, bar, cm, %"}
                  value={cfgStr("unit", "")}
                  onchange={(e) => setCfg("unit", e.currentTarget.value)}
                />
              </td>
            </tr>
            <tr>
              <td>Decimals</td>
              <td>
                <input
                  type="number"
                  min="0"
                  max="8"
                  placeholder={boundTagDef ? String(boundTagDef.decimals ?? 0) : "0"}
                  value={widget.config?.decimals !== undefined ? cfgStr("decimals", "") : ""}
                  onchange={(e) => setCfg("decimals", e.currentTarget.value === "" ? undefined : Number(e.currentTarget.value))}
                />
              </td>
            </tr>
            {#if ["meter", "bar", "tank", "numeric_input"].includes(widget.widget_type)}
              <tr>
                <td>Min / Max</td>
                <td class="pair">
                  <input
                    type="number"
                    placeholder="Min (0)"
                    value={cfgNum("min", 0)}
                    onchange={(e) => setCfg("min", Number(e.currentTarget.value))}
                  />
                  <input
                    type="number"
                    placeholder="Max (100)"
                    value={cfgNum("max", 100)}
                    onchange={(e) => setCfg("max", Number(e.currentTarget.value))}
                  />
                </td>
              </tr>
            {/if}
            {#if ["meter", "bar", "tank"].includes(widget.widget_type)}
              <tr>
                <td>Warn / Alarm limit</td>
                <td class="pair">
                  <input
                    type="number"
                    placeholder="Warn"
                    value={widget.config?.warn !== undefined ? cfgStr("warn", "") : widget.config?.warningAt !== undefined ? cfgStr("warningAt", "") : ""}
                    onchange={(e) => {
                      const val = e.currentTarget.value === "" ? undefined : Number(e.currentTarget.value);
                      setCfg("warn", val);
                      setCfg("warningAt", val);
                    }}
                  />
                  <input
                    type="number"
                    placeholder="Alarm"
                    value={widget.config?.alarm !== undefined ? cfgStr("alarm", "") : widget.config?.alarmAt !== undefined ? cfgStr("alarmAt", "") : ""}
                    onchange={(e) => {
                      const val = e.currentTarget.value === "" ? undefined : Number(e.currentTarget.value);
                      setCfg("alarm", val);
                      setCfg("alarmAt", val);
                    }}
                  />
                </td>
              </tr>
            {/if}
            {#if widget.widget_type === "numeric_input"}
              <tr>
                <td>Variant</td>
                <td>
                  <select
                    value={cfgStr("variant", "stepper")}
                    onchange={(e) => setCfg("variant", e.currentTarget.value)}
                  >
                    <option value="stepper">Stepper (−/+ buttons)</option>
                    <option value="slider">Slider (range bar)</option>
                    <option value="field">Field (direct input)</option>
                  </select>
                </td>
              </tr>
              <tr>
                <td>Step</td>
                <td>
                  <input
                    type="number"
                    min="0.0001"
                    step="any"
                    value={cfgNum("step", 1)}
                    onchange={(e) => setCfg("step", Number(e.currentTarget.value))}
                  />
                </td>
              </tr>
            {/if}
          </tbody>
        </table>
      {/if}

      <!-- Shared typography for text-like controls (not Label — Label has full editor) -->
      {#if ["numeric", "lamp", "write_button", "bool_display", "panel", "bar", "tank", "shape"].includes(widget.widget_type)}
        {@const tc = parseColorAlpha(cfgStr("textColor", "#1f2937"))}
        {@const ttc = parseColorAlpha(cfgStr("titleColor", "#6b7280"))}
        {@const bc = parseColorAlpha(cfgStr("bgColor", "#ffffff"))}
        <table class="props-table">
          <thead>
            <tr><th colspan="2">Appearance · Font & colors</th></tr>
          </thead>
          <tbody>
            {#if widget.widget_type === "write_button" || widget.widget_type === "bool_display"}
              <tr>
                <td>Label</td>
                <td>
                  <input
                    value={cfgStr(widget.widget_type === "write_button" ? "label" : "label", "Label")}
                    onchange={(e) => setCfg("label", e.currentTarget.value)}
                  />
                </td>
              </tr>
            {:else if widget.widget_type !== "shape"}
              <tr>
                <td>Title</td>
                <td>
                  <input
                    value={cfgStr("title", "")}
                    onchange={(e) => setCfg("title", e.currentTarget.value)}
                  />
                </td>
              </tr>
            {:else}
              <tr>
                <td>Title</td>
                <td>
                  <input
                    value={cfgStr("title", "")}
                    onchange={(e) => setCfg("title", e.currentTarget.value)}
                  />
                </td>
              </tr>
            {/if}
            <tr>
              <td>Font</td>
              <td>
                <select
                  value={cfgStr("fontFamily", FONT_OPTIONS[0])}
                  onchange={(e) => setCfg("fontFamily", e.currentTarget.value)}
                >
                  {#each FONT_OPTIONS as f}
                    <option value={f}>{f.split(",")[0]}</option>
                  {/each}
                </select>
              </td>
            </tr>
            <tr>
              <td>Font size</td>
              <td>
                <input
                  type="number"
                  min="6"
                  max="96"
                  value={cfgNum("fontSize", 12)}
                  onchange={(e) => setCfg("fontSize", Number(e.currentTarget.value))}
                />
              </td>
            </tr>
            <tr>
              <td>Weight</td>
              <td>
                <select
                  value={cfgStr("fontWeight", "normal")}
                  onchange={(e) => setCfg("fontWeight", e.currentTarget.value)}
                >
                  <option value="normal">Normal</option>
                  <option value="500">Medium</option>
                  <option value="600">Semi-bold</option>
                  <option value="bold">Bold</option>
                  <option value="800">Extra-bold</option>
                </select>
              </td>
            </tr>
            <tr>
              <td>Text color</td>
              <td>
                <div class="color-picker-box">
                  <div class="color-row">
                    <input
                      type="color"
                      value={tc.hex6}
                      onchange={(e) =>
                        setCfg("textColor", stringifyColorAlpha(e.currentTarget.value, tc.alpha))}
                    />
                    <input
                      type="text"
                      value={cfgStr("textColor", "#1f2937")}
                      onchange={(e) => setCfg("textColor", e.currentTarget.value)}
                    />
                  </div>
                  <div class="alpha-row">
                    <span class="alpha-label">Opacity: {tc.alpha}%</span>
                    <input
                      type="range"
                      min="0"
                      max="100"
                      value={tc.alpha}
                      oninput={(e) =>
                        setCfg("textColor", stringifyColorAlpha(tc.hex6, Number(e.currentTarget.value)))}
                    />
                  </div>
                </div>
              </td>
            </tr>
            {#if widget.widget_type !== "write_button"}
              <tr>
                <td>Title color</td>
                <td>
                  <div class="color-picker-box">
                    <div class="color-row">
                      <input
                        type="color"
                        value={ttc.hex6}
                        onchange={(e) =>
                          setCfg("titleColor", stringifyColorAlpha(e.currentTarget.value, ttc.alpha))}
                      />
                      <input
                        type="text"
                        value={cfgStr("titleColor", "#6b7280")}
                        onchange={(e) => setCfg("titleColor", e.currentTarget.value)}
                      />
                    </div>
                    <div class="alpha-row">
                      <span class="alpha-label">Opacity: {ttc.alpha}%</span>
                      <input
                        type="range"
                        min="0"
                        max="100"
                        value={ttc.alpha}
                        oninput={(e) =>
                          setCfg(
                            "titleColor",
                            stringifyColorAlpha(ttc.hex6, Number(e.currentTarget.value)),
                          )}
                      />
                    </div>
                  </div>
                </td>
              </tr>
            {/if}
            <tr>
              <td>Background</td>
              <td>
                <div class="color-picker-box">
                  <div class="color-row">
                    <input
                      type="color"
                      value={bc.hex6}
                      onchange={(e) =>
                        setCfg("bgColor", stringifyColorAlpha(e.currentTarget.value, bc.alpha))}
                    />
                    <input
                      type="text"
                      value={cfgStr("bgColor", "#ffffff")}
                      onchange={(e) => setCfg("bgColor", e.currentTarget.value)}
                    />
                  </div>
                  <div class="alpha-row">
                    <span class="alpha-label">Opacity: {bc.alpha}%</span>
                    <input
                      type="range"
                      min="0"
                      max="100"
                      value={bc.alpha}
                      oninput={(e) =>
                        setCfg("bgColor", stringifyColorAlpha(bc.hex6, Number(e.currentTarget.value)))}
                    />
                  </div>
                </div>
              </td>
            </tr>
            {#if widget.widget_type === "numeric"}
              <tr>
                <td>Align</td>
                <td>
                  <select
                    value={cfgStr("align", "left")}
                    onchange={(e) => setCfg("align", e.currentTarget.value)}
                  >
                    <option value="left">Left</option>
                    <option value="center">Center</option>
                    <option value="right">Right</option>
                  </select>
                </td>
              </tr>
            {/if}
            {#if widget.widget_type === "lamp"}
              <tr>
                <td>ON label / color</td>
                <td class="pair">
                  <input value={cfgStr("onLabel", "ON")} onchange={(e) => setCfg("onLabel", e.currentTarget.value)} />
                  <input type="color" value={parseColorAlpha(cfgStr("onColor", "#16A34A")).hex6} onchange={(e) => setCfg("onColor", e.currentTarget.value)} />
                </td>
              </tr>
              <tr>
                <td>OFF label / color</td>
                <td class="pair">
                  <input value={cfgStr("offLabel", "OFF")} onchange={(e) => setCfg("offLabel", e.currentTarget.value)} />
                  <input type="color" value={parseColorAlpha(cfgStr("offColor", "#9CA3AF")).hex6} onchange={(e) => setCfg("offColor", e.currentTarget.value)} />
                </td>
              </tr>
            {/if}
            {#if widget.widget_type === "bool_display"}
              <tr>
                <td>TRUE label / color</td>
                <td class="pair">
                  <input value={cfgStr("trueLabel", "TRUE")} onchange={(e) => setCfg("trueLabel", e.currentTarget.value)} />
                  <input type="color" value={parseColorAlpha(cfgStr("trueColor", "#16A34A")).hex6} onchange={(e) => setCfg("trueColor", e.currentTarget.value)} />
                </td>
              </tr>
              <tr>
                <td>FALSE label / color</td>
                <td class="pair">
                  <input value={cfgStr("falseLabel", "FALSE")} onchange={(e) => setCfg("falseLabel", e.currentTarget.value)} />
                  <input type="color" value={parseColorAlpha(cfgStr("falseColor", "#9CA3AF")).hex6} onchange={(e) => setCfg("falseColor", e.currentTarget.value)} />
                </td>
              </tr>
            {/if}
          </tbody>
        </table>
      {/if}

      <!-- LABEL: structured appearance -->
      {#if widget.widget_type === "label"}
        {@const tc = parseColorAlpha(cfgStr("textColor", "#1f2937"))}
        {@const bc = parseColorAlpha(cfgStr("bgColor", "transparent"))}
        {@const brc = parseColorAlpha(cfgStr("borderColor", "transparent"))}
        <table class="props-table">
          <thead>
            <tr><th colspan="2">Label · Text & Font</th></tr>
          </thead>
          <tbody>
            <tr>
              <td>Text</td>
              <td>
                <textarea
                  rows="3"
                  class="text-edit"
                  value={cfgStr("text", "Label")}
                  oninput={(e) => setCfgLive("text", e.currentTarget.value)}
                  onchange={(e) => setCfgLive("text", e.currentTarget.value)}
                ></textarea>
              </td>
            </tr>
            <tr>
              <td>Font</td>
              <td>
                <select
                  value={cfgStr("fontFamily", FONT_OPTIONS[0])}
                  onchange={(e) => setCfg("fontFamily", e.currentTarget.value)}
                >
                  {#each FONT_OPTIONS as f}
                    <option value={f}>{f.split(",")[0]}</option>
                  {/each}
                </select>
              </td>
            </tr>
            <tr>
              <td>Font size</td>
              <td>
                <input
                  type="number"
                  min="6"
                  max="200"
                  value={cfgNum("fontSize", 14)}
                  onchange={(e) => setCfg("fontSize", Number(e.currentTarget.value))}
                />
              </td>
            </tr>
            <tr>
              <td>Weight</td>
              <td>
                <select
                  value={cfgStr("fontWeight", "normal")}
                  onchange={(e) => setCfg("fontWeight", e.currentTarget.value)}
                >
                  <option value="normal">Normal</option>
                  <option value="500">Medium</option>
                  <option value="600">Semi-bold</option>
                  <option value="bold">Bold</option>
                  <option value="800">Extra-bold</option>
                </select>
              </td>
            </tr>
            <tr>
              <td>Style</td>
              <td>
                <select
                  value={cfgStr("fontStyle", "normal")}
                  onchange={(e) => setCfg("fontStyle", e.currentTarget.value)}
                >
                  <option value="normal">Normal</option>
                  <option value="italic">Italic</option>
                </select>
              </td>
            </tr>
            <tr>
              <td>Align H</td>
              <td>
                <select
                  value={cfgStr("align", "left")}
                  onchange={(e) => setCfg("align", e.currentTarget.value)}
                >
                  <option value="left">Left</option>
                  <option value="center">Center</option>
                  <option value="right">Right</option>
                </select>
              </td>
            </tr>
            <tr>
              <td>Align V</td>
              <td>
                <select
                  value={cfgStr("vAlign", "center")}
                  onchange={(e) => setCfg("vAlign", e.currentTarget.value)}
                >
                  <option value="top">Top</option>
                  <option value="center">Center</option>
                  <option value="bottom">Bottom</option>
                </select>
              </td>
            </tr>
            <tr>
              <td>Text color</td>
              <td>
                <div class="color-picker-box">
                  <div class="color-row">
                    <input
                      type="color"
                      value={tc.hex6}
                      onchange={(e) =>
                        setCfg("textColor", stringifyColorAlpha(e.currentTarget.value, tc.alpha))}
                    />
                    <input
                      type="text"
                      value={cfgStr("textColor", "#1f2937")}
                      onchange={(e) => setCfg("textColor", e.currentTarget.value)}
                    />
                  </div>
                  <div class="alpha-row">
                    <span class="alpha-label">Opacity: {tc.alpha}%</span>
                    <input
                      type="range"
                      min="0"
                      max="100"
                      value={tc.alpha}
                      oninput={(e) =>
                        setCfg("textColor", stringifyColorAlpha(tc.hex6, Number(e.currentTarget.value)))}
                    />
                  </div>
                </div>
              </td>
            </tr>
            <tr>
              <td>Background</td>
              <td>
                <div class="color-picker-box">
                  <div class="color-row">
                    <input
                      type="color"
                      value={bc.hex6}
                      onchange={(e) =>
                        setCfg("bgColor", stringifyColorAlpha(e.currentTarget.value, bc.alpha))}
                    />
                    <input
                      type="text"
                      value={cfgStr("bgColor", "transparent")}
                      onchange={(e) => setCfg("bgColor", e.currentTarget.value)}
                    />
                  </div>
                  <div class="alpha-row">
                    <span class="alpha-label">Opacity: {bc.alpha}%</span>
                    <input
                      type="range"
                      min="0"
                      max="100"
                      value={bc.alpha}
                      oninput={(e) =>
                        setCfg("bgColor", stringifyColorAlpha(bc.hex6, Number(e.currentTarget.value)))}
                    />
                  </div>
                </div>
              </td>
            </tr>
            <tr>
              <td>Border</td>
              <td>
                <div class="color-picker-box">
                  <div class="color-row">
                    <input
                      type="color"
                      value={brc.hex6}
                      onchange={(e) =>
                        setCfg("borderColor", stringifyColorAlpha(e.currentTarget.value, brc.alpha))}
                    />
                    <input
                      type="number"
                      min="0"
                      max="20"
                      title="Width px"
                      value={cfgNum("borderWidth", 0)}
                      onchange={(e) => setCfg("borderWidth", Number(e.currentTarget.value))}
                    />
                  </div>
                </div>
              </td>
            </tr>
            <tr>
              <td>Radius</td>
              <td>
                <input
                  type="number"
                  min="0"
                  max="64"
                  value={cfgNum("borderRadius", 0)}
                  onchange={(e) => setCfg("borderRadius", Number(e.currentTarget.value))}
                />
              </td>
            </tr>
          </tbody>
        </table>
      {/if}

      <!-- LINE widget -->
      {#if widget.widget_type === "line"}
        {@const sc = parseColorAlpha(cfgStr("stroke", "#1f2937"))}
        <table class="props-table">
          <thead>
            <tr><th colspan="2">Line · Geometry & style</th></tr>
          </thead>
          <tbody>
            <tr>
              <td>Start X/Y %</td>
              <td class="pair">
                <input
                  type="number"
                  min="0"
                  max="100"
                  step="0.5"
                  value={cfgNum("x1", 5)}
                  onchange={(e) => setCfg("x1", Number(e.currentTarget.value))}
                />
                <input
                  type="number"
                  min="0"
                  max="100"
                  step="0.5"
                  value={cfgNum("y1", 50)}
                  onchange={(e) => setCfg("y1", Number(e.currentTarget.value))}
                />
              </td>
            </tr>
            <tr>
              <td>End X/Y %</td>
              <td class="pair">
                <input
                  type="number"
                  min="0"
                  max="100"
                  step="0.5"
                  value={cfgNum("x2", 95)}
                  onchange={(e) => setCfg("x2", Number(e.currentTarget.value))}
                />
                <input
                  type="number"
                  min="0"
                  max="100"
                  step="0.5"
                  value={cfgNum("y2", 50)}
                  onchange={(e) => setCfg("y2", Number(e.currentTarget.value))}
                />
              </td>
            </tr>
            <tr>
              <td>Stroke color</td>
              <td>
                <div class="color-picker-box">
                  <div class="color-row">
                    <input
                      type="color"
                      value={sc.hex6}
                      onchange={(e) =>
                        setCfg("stroke", stringifyColorAlpha(e.currentTarget.value, sc.alpha))}
                    />
                    <input
                      type="text"
                      value={cfgStr("stroke", "#1f2937")}
                      onchange={(e) => setCfg("stroke", e.currentTarget.value)}
                    />
                  </div>
                  <div class="alpha-row">
                    <span class="alpha-label">Opacity: {sc.alpha}%</span>
                    <input
                      type="range"
                      min="0"
                      max="100"
                      value={sc.alpha}
                      oninput={(e) =>
                        setCfg("stroke", stringifyColorAlpha(sc.hex6, Number(e.currentTarget.value)))}
                    />
                  </div>
                </div>
              </td>
            </tr>
            <tr>
              <td>Thickness</td>
              <td>
                <input
                  type="number"
                  min="0.5"
                  max="40"
                  step="0.5"
                  value={cfgNum("strokeWidth", 2.5)}
                  onchange={(e) => setCfg("strokeWidth", Number(e.currentTarget.value))}
                />
              </td>
            </tr>
            <tr>
              <td>Line style</td>
              <td>
                <select
                  value={cfgStr("lineStyle", "solid")}
                  onchange={(e) => setCfg("lineStyle", e.currentTarget.value)}
                >
                  <option value="solid">Solid ────</option>
                  <option value="dashed">Dashed - - -</option>
                  <option value="dotted">Dotted · · ·</option>
                  <option value="dashdot">Dash-dot -·-</option>
                  <option value="longdash">Long dash —— ——</option>
                </select>
              </td>
            </tr>
            <tr>
              <td>Start cap</td>
              <td>
                <select
                  value={cfgStr("startCap", "none")}
                  onchange={(e) => setCfg("startCap", e.currentTarget.value)}
                >
                  <option value="none">None</option>
                  <option value="arrow">Arrow ▶</option>
                  <option value="open-arrow">Open arrow ></option>
                  <option value="circle">Circle ●</option>
                  <option value="square">Square ■</option>
                  <option value="diamond">Diamond ◆</option>
                  <option value="bar">Bar |</option>
                </select>
              </td>
            </tr>
            <tr>
              <td>End cap</td>
              <td>
                <select
                  value={cfgStr("endCap", "arrow")}
                  onchange={(e) => setCfg("endCap", e.currentTarget.value)}
                >
                  <option value="none">None</option>
                  <option value="arrow">Arrow ▶</option>
                  <option value="open-arrow">Open arrow ></option>
                  <option value="circle">Circle ●</option>
                  <option value="square">Square ■</option>
                  <option value="diamond">Diamond ◆</option>
                  <option value="bar">Bar |</option>
                </select>
              </td>
            </tr>
            <tr>
              <td>Cap size</td>
              <td>
                <input
                  type="number"
                  min="4"
                  max="48"
                  value={cfgNum("capSize", 12)}
                  onchange={(e) => setCfg("capSize", Number(e.currentTarget.value))}
                />
              </td>
            </tr>
          </tbody>
        </table>
        <p class="line-hint">
          Design: niebieska kropka = start, zielona = koniec — przeciągnij końce. Ramka widgetu skaluje linię.
        </p>
      {/if}

      <!-- IMAGE WIDGET -->
      {#if widget.widget_type === "image"}
        {@const projectImages = collectProjectImages($project?.tree ?? [])}
        {@const currentSrc = cfgStr("src", "")}
        {@const currentTrueSrc = cfgStr("trueSrc", "")}
        {@const stateMode = cfgStr("stateMode", "none")}
        <input
          type="file"
          accept="image/*,.svg,.png,.jpg,.jpeg,.gif,.webp"
          multiple
          bind:this={propsFileInputEl}
          onchange={handlePropsImageUpload}
          style="display:none;"
        />
        <table class="props-table">
          <thead>
            <tr><th colspan="2">Image / Graphic Properties</th></tr>
          </thead>
          <tbody>
            <tr>
              <td>Import Image</td>
              <td>
                <button
                  type="button"
                  style="width:100%;padding:4px 8px;cursor:pointer;background:#094771;border:1px solid #007acc;color:#fff;border-radius:3px;font-size:11px;"
                  onclick={triggerPropsImageUpload}
                >
                  🖼️ Add Image from Disk...
                </button>
              </td>
            </tr>

            <!-- Primary Image (Default / FALSE) -->
            <tr>
              <td>Project Image (Default)</td>
              <td>
                <select
                  value={projectImages.find((img) => img.content === currentSrc)?.id ?? ""}
                  onchange={(e) => {
                    const sel = projectImages.find((img) => img.id === e.currentTarget.value);
                    if (sel) setCfg("src", sel.content);
                  }}
                >
                  <option value="">(Custom URL / SVG or default)</option>
                  {#each projectImages as img (img.id)}
                    <option value={img.id}>{img.path}</option>
                  {/each}
                </select>
              </td>
            </tr>
            <tr>
              <td>Source / URL (Default)</td>
              <td>
                <input
                  type="text"
                  placeholder="Leave empty for default pump SVG"
                  value={cfgStr("src", "")}
                  onchange={(e) => setCfg("src", e.currentTarget.value)}
                />
              </td>
            </tr>

            <!-- 2-State Image Switching -->
            <tr>
              <td colspan="2" class="section">2-State Image Switching</td>
            </tr>
            <tr>
              <td>Condition Mode</td>
              <td>
                <select
                  value={stateMode}
                  onchange={(e) => setCfg("stateMode", e.currentTarget.value)}
                >
                  <option value="none">None (off - single image)</option>
                  <option value="tag_true">Bit / BOOL = true (1)</option>
                  <option value="tag_false">Bit / BOOL = false (0)</option>
                  <option value="tag_bit">Register bit (N)</option>
                  <option value="tag_val_eq">Register value ==</option>
                  <option value="tag_val_gt">Register value ></option>
                  <option value="tag_val_lt">Register value &lt;</option>
                  <option value="tag_val_neq">Register value !=</option>
                </select>
              </td>
            </tr>

            {#if stateMode !== "none"}
              <tr>
                <td>Tag for state switch</td>
                <td>
                  <select
                    value={cfgStr("stateTagId", "")}
                    onchange={(e) => setCfg("stateTagId", e.currentTarget.value)}
                  >
                    <option value="">(Use widget main tag)</option>
                    {#each tags as t (t.id)}
                      <option value={t.id}>{t.name} ({t.id})</option>
                    {/each}
                  </select>
                </td>
              </tr>
              {#if stateMode === "tag_bit"}
                <tr>
                  <td>Bit Index (0..15)</td>
                  <td>
                    <input
                      type="number"
                      min="0"
                      max="15"
                      value={cfgNum("stateBit", 0)}
                      onchange={(e) => setCfg("stateBit", Number(e.currentTarget.value))}
                    />
                  </td>
                </tr>
              {/if}
              {#if ["tag_val_eq", "tag_val_gt", "tag_val_lt", "tag_val_neq"].includes(stateMode)}
                <tr>
                  <td>Target Value</td>
                  <td>
                    <input
                      type="number"
                      value={cfgNum("stateVal", 1)}
                      onchange={(e) => setCfg("stateVal", Number(e.currentTarget.value))}
                    />
                  </td>
                </tr>
              {/if}

              <tr>
                <td>Project Image (TRUE)</td>
                <td>
                  <select
                    value={projectImages.find((img) => img.content === currentTrueSrc)?.id ?? ""}
                    onchange={(e) => {
                      const sel = projectImages.find((img) => img.id === e.currentTarget.value);
                      if (sel) setCfg("trueSrc", sel.content);
                    }}
                  >
                    <option value="">(Select graphic for TRUE state)</option>
                    {#each projectImages as img (img.id)}
                      <option value={img.id}>{img.path}</option>
                    {/each}
                  </select>
                </td>
              </tr>
              <tr>
                <td>Source (TRUE state)</td>
                <td>
                  <input
                    type="text"
                    placeholder="Image source for TRUE state"
                    value={cfgStr("trueSrc", "")}
                    onchange={(e) => setCfg("trueSrc", e.currentTarget.value)}
                  />
                </td>
              </tr>
            {/if}

            <tr>
              <td colspan="2" class="section">Display Options</td>
            </tr>
            <tr>
              <td>Object Fit</td>
              <td>
                <select
                  value={cfgStr("fit", "contain")}
                  onchange={(e) => setCfg("fit", e.currentTarget.value)}
                >
                  <option value="contain">Contain (Keep aspect)</option>
                  <option value="cover">Cover (Fill & Crop)</option>
                  <option value="fill">Fill (Stretch)</option>
                  <option value="none">None (Original size)</option>
                </select>
              </td>
            </tr>
            <tr>
              <td>Alt Text</td>
              <td>
                <input
                  type="text"
                  value={cfgStr("alt", "Process image")}
                  onchange={(e) => setCfg("alt", e.currentTarget.value)}
                />
              </td>
            </tr>
          </tbody>
        </table>
      {/if}

      <!-- EMBEDDED SCREEN / FACEPLATE -->
      {#if widget.widget_type === "embedded_screen"}
        {@const targetId = cfgStr("target_form_id", "")}
        {@const overrides = (widget.config?.tag_overrides ?? {}) as Record<string, string>}
        <table class="props-table">
          <thead>
            <tr><th colspan="2">Embedded Screen / Faceplate</th></tr>
          </thead>
          <tbody>
            <tr>
              <td>Target Screen</td>
              <td>
                <div class="target-screen-row">
                  <select
                    value={targetId}
                    onchange={(e) => setCfg("target_form_id", e.currentTarget.value)}
                  >
                    <option value="">-- Select Master Screen --</option>
                    {#each ($project?.forms ?? []) as f (f.id)}
                      {#if f.id !== form?.id}
                        <option value={f.id}>{f.name} ({f.width}×{f.height})</option>
                      {/if}
                    {/each}
                  </select>
                  {#if targetId}
                    <button
                      class="btn-open-screen"
                      title="Open target screen in Designer"
                      onclick={() => selectedFormId.set(targetId)}
                    >
                      ✏️ Edit
                    </button>
                  {/if}
                </div>
              </td>
            </tr>
            <tr>
              <td>Tag Prefix</td>
              <td>
                <input
                  type="text"
                  placeholder="e.g. PUMP1_ or TK1_"
                  value={cfgStr("tag_prefix", "")}
                  onchange={(e) => setCfg("tag_prefix", e.currentTarget.value)}
                />
              </td>
            </tr>
            <tr>
              <td>Scale Mode</td>
              <td>
                <select
                  value={cfgStr("scale_mode", "fit")}
                  onchange={(e) => setCfg("scale_mode", e.currentTarget.value)}
                >
                  <option value="fit">Fit (Proportional Scale)</option>
                  <option value="stretch">Stretch (Fill Container)</option>
                  <option value="clip">Clip (1:1 Overflow Hidden)</option>
                  <option value="scroll">Scroll (1:1 Scrollbars)</option>
                </select>
              </td>
            </tr>
          </tbody>
        </table>

        <!-- Tag Overrides Table Editor -->
        <table class="props-table tag-overrides-table">
          <thead>
            <tr>
              <th colspan="3">Tag Overrides / Mapping</th>
            </tr>
          </thead>
          <tbody>
            {#each Object.entries(overrides) as [key, val]}
              <tr>
                <td>
                  <input
                    type="text"
                    placeholder="Inner Tag / Key"
                    value={key}
                    onchange={(e) => {
                      const newKey = e.currentTarget.value.trim();
                      const next = { ...overrides };
                      delete next[key];
                      if (newKey) next[newKey] = val;
                      setCfg("tag_overrides", next);
                    }}
                  />
                </td>
                <td>
                  <select
                    value={val}
                    onchange={(e) => {
                      const next = { ...overrides, [key]: e.currentTarget.value };
                      setCfg("tag_overrides", next);
                    }}
                  >
                    <option value="">-- Direct PLC Tag --</option>
                    {#each tags as t (t.id)}
                      <option value={t.id}>{t.name || t.id}</option>
                    {/each}
                  </select>
                </td>
                <td style="width: 24px; text-align: center;">
                  <button
                    class="btn-del-rule"
                    title="Remove Rule"
                    onclick={() => {
                      const next = { ...overrides };
                      delete next[key];
                      setCfg("tag_overrides", next);
                    }}
                  >
                    ✕
                  </button>
                </td>
              </tr>
            {/each}
            <tr>
              <td colspan="3" style="text-align: right; padding: 4px 6px;">
                <button
                  class="btn-add-override"
                  onclick={() => {
                    const newKey = `TAG_${Object.keys(overrides).length + 1}`;
                    setCfg("tag_overrides", { ...overrides, [newKey]: "" });
                  }}
                >
                  + Add Tag Override
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      {/if}

      <!-- GENERIC DYNAMICS (all widgets) -->
      <ConditionEditor
        title="Custom animation"
        mode={cfgStr("animationMode", "none")}
        tagId={cfgStr("animationTagId", "")}
        bit={cfgNum("animationBit", 0)}
        val={cfgNum("animationVal", 1)}
        {tags}
        onMode={(v) => setCfg("animationMode", v)}
        onTag={(v) => setCfg("animationTagId", v)}
        onBit={(v) => setCfg("animationBit", v)}
        onVal={(v) => setCfg("animationVal", v)}
      />
      <ConditionEditor
        title="Blink"
        mode={cfgStr("blinkMode", "none")}
        tagId={cfgStr("blinkTagId", "")}
        bit={cfgNum("blinkBit", 0)}
        val={cfgNum("blinkVal", 1)}
        {tags}
        onMode={(v) => setCfg("blinkMode", v)}
        onTag={(v) => setCfg("blinkTagId", v)}
        onBit={(v) => setCfg("blinkBit", v)}
        onVal={(v) => setCfg("blinkVal", v)}
      />
      <table class="props-table">
        <tbody>
          <tr>
            <td>Blink speed (ms)</td>
            <td>
              <input
                type="number"
                min="100"
                max="5000"
                step="50"
                value={cfgNum("blinkSpeedMs", 600)}
                onchange={(e) => setCfg("blinkSpeedMs", Number(e.currentTarget.value))}
              />
            </td>
          </tr>
        </tbody>
      </table>

      {#if widget.widget_type === "label"}
        <ConditionEditor
          title="Marquee / scroll text (train)"
          mode={cfgStr("scrollMode", "none")}
          tagId={cfgStr("scrollTagId", "")}
          bit={cfgNum("scrollBit", 0)}
          val={cfgNum("scrollVal", 1)}
          {tags}
          onMode={(v) => setCfg("scrollMode", v)}
          onTag={(v) => setCfg("scrollTagId", v)}
          onBit={(v) => setCfg("scrollBit", v)}
          onVal={(v) => setCfg("scrollVal", v)}
        />
        <table class="props-table">
          <tbody>
            <tr>
              <td>Scroll speed (s)</td>
              <td>
                <input
                  type="number"
                  min="1"
                  max="60"
                  step="0.5"
                  value={cfgNum("scrollSpeedSec", 8)}
                  onchange={(e) => setCfg("scrollSpeedSec", Number(e.currentTarget.value))}
                />
              </td>
            </tr>
            <tr>
              <td>Scroll direction</td>
              <td>
                <select
                  value={cfgStr("scrollDir", "left")}
                  onchange={(e) => setCfg("scrollDir", e.currentTarget.value)}
                >
                  <option value="left">← Left (train)</option>
                  <option value="right">→ Right</option>
                </select>
              </td>
            </tr>
          </tbody>
        </table>
      {/if}

      <ConditionEditor
        title="Visibility (show / hide)"
        mode={cfgStr("visibilityMode", "always")}
        tagId={cfgStr("visibilityTagId", "")}
        bit={cfgNum("visibilityBit", 0)}
        val={cfgNum("visibilityVal", 1)}
        {tags}
        onMode={(v) => setCfg("visibilityMode", v)}
        onTag={(v) => setCfg("visibilityTagId", v)}
        onBit={(v) => setCfg("visibilityBit", v)}
        onVal={(v) => setCfg("visibilityVal", v)}
      />

      <!-- Raw remaining keys for other widget types -->
      {#if rawConfigEntries.length > 0}
        <table class="props-table">
          <thead>
            <tr><th colspan="2">Other properties</th></tr>
          </thead>
          <tbody>
            {#each rawConfigEntries as [key, val]}
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
                            const newColor = stringifyColorAlpha(
                              e.currentTarget.value,
                              colorInfo.alpha,
                            );
                            setCfg(key, newColor);
                          }}
                        />
                        <input
                          type="text"
                          value={String(val ?? "")}
                          placeholder="#RRGGBBAA"
                          onchange={(e) => setCfg(key, e.currentTarget.value)}
                        />
                      </div>
                      <div class="alpha-row">
                        <span class="alpha-label">Opacity: {colorInfo.alpha}%</span>
                        <input
                          type="range"
                          min="0"
                          max="100"
                          value={colorInfo.alpha}
                          oninput={(e) => {
                            const a = Number(e.currentTarget.value);
                            setCfg(key, stringifyColorAlpha(colorInfo.hex6, a));
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
      {/if}
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
  .pair {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px;
  }
  .line-hint {
    margin: 0;
    padding: 6px 10px 10px;
    font-size: 10px;
    color: #9d9d9d;
    line-height: 1.35;
  }
  .text-edit {
    width: 100%;
    min-height: 52px;
    resize: vertical;
    box-sizing: border-box;
    background: #3c3c3c;
    border: 1px solid #555;
    color: #eee;
    border-radius: 3px;
    padding: 6px;
    font-family: inherit;
    font-size: 12px;
  }
  .text-edit:focus {
    border-color: #007acc;
    outline: none;
  }
  :global(td.section) {
    font-size: 10px;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: #9d9d9d;
    padding-top: 10px !important;
    background: #2a2a2c;
  }
  .target-screen-row {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .target-screen-row select {
    flex: 1;
  }
  .btn-open-screen {
    background: #2563eb;
    color: #ffffff;
    border: none;
    border-radius: 3px;
    padding: 3px 6px;
    font-size: 10px;
    font-weight: 700;
    cursor: pointer;
    white-space: nowrap;
  }
  .btn-open-screen:hover {
    background: #1d4ed8;
  }
  .tag-overrides-table td input[type="text"] {
    width: 100%;
    box-sizing: border-box;
  }
  .btn-del-rule {
    background: transparent;
    color: #ef4444;
    border: none;
    font-weight: bold;
    cursor: pointer;
    font-size: 11px;
    padding: 2px 4px;
  }
  .btn-del-rule:hover {
    color: #f87171;
  }
  .btn-add-override {
    background: #1e293b;
    border: 1px dashed #475569;
    color: #38bdf8;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 700;
    padding: 3px 8px;
    cursor: pointer;
  }
  .btn-add-override:hover {
    background: #334155;
    border-color: #38bdf8;
  }
</style>
