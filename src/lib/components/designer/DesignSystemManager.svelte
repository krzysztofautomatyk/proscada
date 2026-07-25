<script lang="ts">
  import type {
    AnimationKind,
    ProjectAnimationPreset,
    ProjectFontToken,
    ProjectStyleClass,
  } from "$lib/types";
  import { project, updateProjectDesignSystem } from "$lib/stores/app";
  import {
    defaultProjectDesignSystem,
    normalizeProjectDesignSystem,
  } from "$lib/utils/designSystem";

  const system = $derived(normalizeProjectDesignSystem($project?.design_system));

  function updateFont(id: string, patch: Partial<ProjectFontToken>) {
    updateProjectDesignSystem({
      ...system,
      fonts: system.fonts.map((font) => (font.id === id ? { ...font, ...patch } : font)),
    });
  }

  function updateStyle(id: string, patch: Partial<ProjectStyleClass>) {
    updateProjectDesignSystem({
      ...system,
      styles: system.styles.map((style) => (style.id === id ? { ...style, ...patch } : style)),
    });
  }

  function updateAnimation(id: string, patch: Partial<ProjectAnimationPreset>) {
    updateProjectDesignSystem({
      ...system,
      animations: system.animations.map((animation) =>
        animation.id === id ? { ...animation, ...patch } : animation,
      ),
    });
  }

  function addFont() {
    const id = `font-${Date.now().toString(36)}`;
    updateProjectDesignSystem({
      ...system,
      fonts: [
        ...system.fonts,
        {
          id,
          name: "New Font Role",
          family: "Segoe UI",
          fallback: "system-ui, sans-serif",
          size: 12,
          weight: "600",
          lineHeight: 1.2,
        },
      ],
    });
  }

  function addStyle() {
    const id = `style-${Date.now().toString(36)}`;
    updateProjectDesignSystem({
      ...system,
      styles: [
        ...system.styles,
        {
          id,
          name: "New Style",
          target: "*",
          surface: "#ffffff",
          text: "#1f2937",
          accent: "#2563eb",
          border: "#cbd5e1",
        },
      ],
    });
  }

  function addAnimation() {
    const id = `anim-${Date.now().toString(36)}`;
    updateProjectDesignSystem({
      ...system,
      animations: [
        ...system.animations,
        { id, name: "New Animation", kind: "pulse", durationMs: 1200, easing: "ease-in-out" },
      ],
    });
  }

  function remove(kind: "fonts" | "styles" | "animations", id: string) {
    if (system[kind].length <= 1) return;
    updateProjectDesignSystem({ ...system, [kind]: system[kind].filter((item) => item.id !== id) });
  }
</script>

<div class="manager">
  <header>
    <div><strong>Project Design System</strong><span>Theme · fonts · styles · animations</span></div>
    <button type="button" onclick={() => updateProjectDesignSystem(defaultProjectDesignSystem())}>Reset</button>
  </header>

  <section>
    <div class="section-title"><span>Font Tokens ({system.fonts.length})</span><button type="button" onclick={addFont}>+</button></div>
    {#each system.fonts as font (font.id)}
      <article>
        <input aria-label="Font token name" value={font.name} onchange={(event) => updateFont(font.id, { name: event.currentTarget.value })} />
        <input aria-label="Font family" value={font.family} onchange={(event) => updateFont(font.id, { family: event.currentTarget.value })} />
        <div class="row">
          <input aria-label="Font size" type="number" min="6" max="96" value={font.size} onchange={(event) => updateFont(font.id, { size: Number(event.currentTarget.value) })} />
          <select aria-label="Font weight" value={font.weight} onchange={(event) => updateFont(font.id, { weight: event.currentTarget.value })}>
            <option value="400">Regular</option><option value="600">Semi-bold</option><option value="700">Bold</option><option value="800">Extra-bold</option>
          </select>
          <button class="delete" type="button" disabled={system.fonts.length <= 1} onclick={() => remove("fonts", font.id)}>×</button>
        </div>
        <small>{font.id} · fallback {font.fallback}</small>
      </article>
    {/each}
  </section>

  <section>
    <div class="section-title"><span>Style Classes ({system.styles.length})</span><button type="button" onclick={addStyle}>+</button></div>
    {#each system.styles as style (style.id)}
      <article>
        <input aria-label="Style name" value={style.name} onchange={(event) => updateStyle(style.id, { name: event.currentTarget.value })} />
        <input aria-label="Style targets" value={style.target} onchange={(event) => updateStyle(style.id, { target: event.currentTarget.value })} />
        <div class="colors">
          <label title="Surface"><input type="color" value={style.surface} onchange={(event) => updateStyle(style.id, { surface: event.currentTarget.value })} />BG</label>
          <label title="Text"><input type="color" value={style.text} onchange={(event) => updateStyle(style.id, { text: event.currentTarget.value })} />TXT</label>
          <label title="Accent"><input type="color" value={style.accent} onchange={(event) => updateStyle(style.id, { accent: event.currentTarget.value })} />ACC</label>
          <label title="Border"><input type="color" value={style.border} onchange={(event) => updateStyle(style.id, { border: event.currentTarget.value })} />BRD</label>
          <button class="delete" type="button" disabled={system.styles.length <= 1} onclick={() => remove("styles", style.id)}>×</button>
        </div>
        <small>{style.id}</small>
      </article>
    {/each}
  </section>

  <section>
    <div class="section-title"><span>Animation Presets ({system.animations.length})</span><button type="button" onclick={addAnimation}>+</button></div>
    {#each system.animations as animation (animation.id)}
      <article>
        <input aria-label="Animation name" value={animation.name} onchange={(event) => updateAnimation(animation.id, { name: event.currentTarget.value })} />
        <div class="row">
          <select aria-label="Animation kind" value={animation.kind} onchange={(event) => updateAnimation(animation.id, { kind: event.currentTarget.value as AnimationKind })}>
            <option value="none">None</option><option value="pulse">Pulse</option><option value="rotate">Rotate</option><option value="fade">Fade</option><option value="slide">Slide</option>
          </select>
          <input aria-label="Animation duration" type="number" min="250" max="10000" step="50" value={animation.durationMs} onchange={(event) => updateAnimation(animation.id, { durationMs: Number(event.currentTarget.value) })} />
          <button class="delete" type="button" disabled={system.animations.length <= 1} onclick={() => remove("animations", animation.id)}>×</button>
        </div>
        <small>{animation.id} · {animation.easing}</small>
      </article>
    {/each}
  </section>
</div>

<style>
  .manager { height: 100%; overflow: auto; background: var(--vs-panel, #252526); color: var(--vs-text, #ccc); font-size: 10px; }
  header { position: sticky; top: 0; z-index: 2; display: flex; justify-content: space-between; gap: 8px; padding: 9px; border-bottom: 1px solid var(--vs-border, #444); background: #202020; }
  header strong, header span { display: block; } header strong { color: #fff; font-size: 11px; } header span { margin-top: 2px; color: #9ca3af; font-size: 9px; }
  section { padding: 7px; border-bottom: 1px solid var(--vs-border, #444); }
  .section-title { display: flex; align-items: center; justify-content: space-between; margin-bottom: 5px; color: #93c5fd; font-weight: 800; text-transform: uppercase; }
  article { display: grid; gap: 4px; margin-bottom: 6px; padding: 6px; border: 1px solid #444; border-radius: 4px; background: #2d2d30; }
  input, select, button { min-width: 0; box-sizing: border-box; border: 1px solid #555; border-radius: 3px; background: #1e1e1e; color: #ddd; font: inherit; }
  input, select { width: 100%; min-height: 24px; padding: 3px 5px; } button { min-height: 23px; padding: 2px 7px; cursor: pointer; }
  .row { display: grid; grid-template-columns: 1fr 1fr auto; gap: 4px; }
  .colors { display: flex; align-items: center; gap: 4px; } .colors label { display: grid; gap: 1px; color: #9ca3af; font-size: 7px; text-align: center; } .colors input { width: 32px; padding: 1px; }
  .delete { color: #fca5a5; } small { color: #8b949e; font-size: 8px; }
</style>

