<script lang="ts">
  import type { AlarmDefinition, AlarmGroupDefinition, AlarmPriority } from "$lib/types";
  import { dirty, log, project } from "$lib/stores/app";

  const groups = $derived($project?.alarm_groups ?? []);
  const alarms = $derived($project?.alarms ?? []);
  const tags = $derived($project?.tags ?? []);

  function updateProject(patch: { alarm_groups?: AlarmGroupDefinition[]; alarms?: AlarmDefinition[] }) {
    project.update((current) => (current ? { ...current, ...patch } : current));
    dirty.set(true);
  }

  function addGroup() {
    const id = `alarm-group-${Date.now().toString(36)}`;
    updateProject({
      alarm_groups: [
        ...groups,
        { id, name: "New Alarm Group", parent_id: null, object_id: null, description: "" },
      ],
    });
    log(`Alarm group created: ${id}`, "ok");
  }

  function updateGroup(id: string, patch: Partial<AlarmGroupDefinition>) {
    updateProject({ alarm_groups: groups.map((group) => (group.id === id ? { ...group, ...patch } : group)) });
  }

  function deleteGroup(id: string) {
    if (alarms.some((alarm) => alarm.group_id === id)) {
      alert("Move or delete alarms assigned to this group first.");
      return;
    }
    updateProject({ alarm_groups: groups.filter((group) => group.id !== id) });
  }

  function addAlarm() {
    if (!tags[0]) {
      alert("Create a tag before adding an alarm.");
      return;
    }
    const alarm: AlarmDefinition = {
      id: `alarm-${Date.now().toString(36)}`,
      name: "New Alarm",
      tag_id: tags[0].id,
      group_id: groups[0]?.id ?? "",
      priority: "medium",
      when_true: true,
      hi_limit: null,
      lo_limit: null,
      deadband: 0,
      on_delay_ms: 0,
      off_delay_ms: 0,
      latching: false,
      message: "Operator action required",
    };
    updateProject({ alarms: [...alarms, alarm] });
    log(`Alarm created: ${alarm.id}`, "ok");
  }

  function updateAlarm(id: string, patch: Partial<AlarmDefinition>) {
    updateProject({ alarms: alarms.map((alarm) => (alarm.id === id ? { ...alarm, ...patch } : alarm)) });
  }
</script>

<div class="manager">
  <header><strong>Central Alarm Manager</strong><span>Groups · definitions · lifecycle parameters</span></header>
  <section>
    <div class="section-title"><span>Alarm Groups ({groups.length})</span><button type="button" onclick={addGroup}>+</button></div>
    {#each groups as group (group.id)}
      <article>
        <input aria-label="Alarm group name" value={group.name} onchange={(event) => updateGroup(group.id, { name: event.currentTarget.value })} />
        <div class="row">
          <input aria-label="Object ID" placeholder="objectId" value={group.object_id ?? ""} onchange={(event) => updateGroup(group.id, { object_id: event.currentTarget.value || null })} />
          <select aria-label="Parent alarm group" value={group.parent_id ?? ""} onchange={(event) => updateGroup(group.id, { parent_id: event.currentTarget.value || null })}>
            <option value="">(root)</option>
            {#each groups.filter((candidate) => candidate.id !== group.id) as candidate}<option value={candidate.id}>{candidate.name}</option>{/each}
          </select>
          <button class="delete" type="button" onclick={() => deleteGroup(group.id)}>×</button>
        </div>
        <small>{group.id}</small>
      </article>
    {/each}
  </section>
  <section>
    <div class="section-title"><span>Alarm Definitions ({alarms.length})</span><button type="button" onclick={addAlarm}>+</button></div>
    {#each alarms as alarm (alarm.id)}
      <article>
        <input aria-label="Alarm name" value={alarm.name} onchange={(event) => updateAlarm(alarm.id, { name: event.currentTarget.value })} />
        <input aria-label="Alarm message" value={alarm.message} onchange={(event) => updateAlarm(alarm.id, { message: event.currentTarget.value })} />
        <div class="row two">
          <select aria-label="Alarm tag" value={alarm.tag_id} onchange={(event) => updateAlarm(alarm.id, { tag_id: event.currentTarget.value })}>{#each tags as tag}<option value={tag.id}>{tag.name}</option>{/each}</select>
          <select aria-label="Alarm group" value={alarm.group_id ?? ""} onchange={(event) => updateAlarm(alarm.id, { group_id: event.currentTarget.value })}><option value="">(ungrouped)</option>{#each groups as group}<option value={group.id}>{group.name}</option>{/each}</select>
        </div>
        <div class="row three">
          <select aria-label="Alarm priority" value={alarm.priority} onchange={(event) => updateAlarm(alarm.id, { priority: event.currentTarget.value as AlarmPriority })}><option value="low">Low</option><option value="medium">Medium</option><option value="high">High</option><option value="critical">Critical</option></select>
          <label><input type="checkbox" checked={alarm.latching ?? false} onchange={(event) => updateAlarm(alarm.id, { latching: event.currentTarget.checked })} /> Latch</label>
          <button class="delete" type="button" onclick={() => updateProject({ alarms: alarms.filter((item) => item.id !== alarm.id) })}>×</button>
        </div>
        <div class="timing">
          <label>Deadband<input type="number" min="0" value={alarm.deadband ?? 0} onchange={(event) => updateAlarm(alarm.id, { deadband: Number(event.currentTarget.value) })} /></label>
          <label>ON delay ms<input type="number" min="0" value={alarm.on_delay_ms ?? 0} onchange={(event) => updateAlarm(alarm.id, { on_delay_ms: Number(event.currentTarget.value) })} /></label>
          <label>OFF delay ms<input type="number" min="0" value={alarm.off_delay_ms ?? 0} onchange={(event) => updateAlarm(alarm.id, { off_delay_ms: Number(event.currentTarget.value) })} /></label>
        </div>
        <small>{alarm.id}</small>
      </article>
    {/each}
  </section>
  <footer>Alarm Panel controls subscribe to runtime alarm instances. ACK, latching reset, deadband and delays are enforced by the Rust engine.</footer>
</div>

<style>
  .manager { height: 100%; overflow: auto; background: var(--vs-panel, #252526); color: #ccc; font-size: 9px; }
  header { padding: 9px; border-bottom: 1px solid #444; background: #202020; } header strong, header span { display: block; } header strong { color: #fff; font-size: 11px; } header span { margin-top: 2px; color: #9ca3af; }
  section { padding: 7px; border-bottom: 1px solid #444; } .section-title { display: flex; justify-content: space-between; align-items: center; margin-bottom: 5px; color: #fca5a5; font-weight: 800; text-transform: uppercase; }
  article { display: grid; gap: 4px; margin-bottom: 6px; padding: 6px; border: 1px solid #444; border-radius: 4px; background: #2d2d30; }
  input, select, button { min-width: 0; min-height: 24px; box-sizing: border-box; border: 1px solid #555; border-radius: 3px; background: #1e1e1e; color: #ddd; padding: 3px 5px; font: inherit; } button { cursor: pointer; }
  .row { display: grid; grid-template-columns: 1fr 1fr auto; gap: 4px; } .row.two { grid-template-columns: 1fr 1fr; } .row.three { grid-template-columns: 1fr 70px auto; } .row label { display: flex; align-items: center; gap: 3px; } .row label input { min-height: auto; }
  .timing { display: grid; grid-template-columns: repeat(3, 1fr); gap: 3px; } .timing label { display: grid; gap: 2px; color: #9ca3af; } .delete { color: #fca5a5; }
  small, footer { color: #8b949e; font-size: 8px; } footer { padding: 8px; line-height: 1.4; }
</style>

