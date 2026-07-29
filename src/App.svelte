<script lang="ts">
  import { onMount } from "svelte";
  import {
    project,
    snapshot,
    mode,
    activeForm,
    selectedWidget,
    selectedFormId,
    selectedWidgetId,
    selectedWidgetIds,
    selectedSolutionNode,
    selectSolutionNode,
    isMainScreen,
    tagMap,
    audit,
    dirty,
    initApp,
    connectDevice,
    disconnectDevice,
    switchMode,
    persistProject,
    deleteSelectedWidget,
    addNewForm,
    deleteForm,
    refreshAudit,
    log,
    cutSelectedWidgets,
    copySelectedWidgets,
    pasteWidgets,
    duplicateSelected,
    multiCopySelected,
    groupSelectedWidgets,
    ungroupSelectedWidgets,
    selectAllWidgets,
    toggleLockSelected,
    openAttributesPanel,
    focusPropertiesTick,
    alignSelectedWidgets,
    moveSelectedWidgets,
    undoAction,
    redoAction,
    startWindowOpen,
  } from "$lib/stores/app";
  import { api } from "$lib/services/api";
  import type { LeftPanelTab, Role } from "$lib/types";
  import SolutionExplorer from "$lib/components/shell/SolutionExplorer.svelte";
  import MenuBar from "$lib/components/shell/MenuBar.svelte";
  import Toolbox from "$lib/components/designer/Toolbox.svelte";
  import Properties from "$lib/components/designer/Properties.svelte";
  import ObjectList from "$lib/components/designer/ObjectList.svelte";
  import DesignerCanvas from "$lib/components/designer/DesignerCanvas.svelte";
  import OutputPanel from "$lib/components/shell/OutputPanel.svelte";
  import DocumentEditor from "$lib/components/shell/DocumentEditor.svelte";
  import VariablesEditor from "$lib/components/shell/VariablesEditor.svelte";
  import DesignSystemManager from "$lib/components/designer/DesignSystemManager.svelte";
  import ComponentLibraryManager from "$lib/components/designer/ComponentLibraryManager.svelte";
  import AlarmManagerEditor from "$lib/components/designer/AlarmManagerEditor.svelte";
  import SettingsModal from "$lib/components/shell/SettingsModal.svelte";
  import StartWindow from "$lib/components/shell/StartWindow.svelte";
  import AddDeviceModal from "$lib/components/shell/AddDeviceModal.svelte";
  import AddAlarmModal from "$lib/components/shell/AddAlarmModal.svelte";
  import AddVariableModal from "$lib/components/shell/AddVariableModal.svelte";
  import { addDeviceModalOpen, addAlarmModalOpen, addVariableModalOpen } from "$lib/stores/app";
  import { ensureProjectTree, isDocKind, iconFor } from "$lib/utils/projectTree";
  import { appSettings } from "$lib/stores/settings";
  import { validateProject } from "$lib/utils/validation";
  import { activate, resizeOnKey } from "$lib/utils/a11y";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import LoginModal from "$lib/components/auth/LoginModal.svelte";
  import PinChallengeModal from "$lib/components/auth/PinChallengeModal.svelte";
  import UserManagementModal from "$lib/components/auth/UserManagementModal.svelte";
  import ChangePasswordModal from "$lib/components/auth/ChangePasswordModal.svelte";

  let leftTab = $state<LeftPanelTab>("solution");
  let settingsOpen = $state(false);
  let loginModalOpen = $state(false);
  let pinChallengeOpen = $state(false);
  let userMgmtModalOpen = $state(false);
  let pinChallengeAction = $state("Potwierdzenie operacji");
  let pendingWriteFn = $state<(() => void) | null>(null);

  let propertiesEl = $state<HTMLDivElement | null>(null);
  let workspaceEl = $state<HTMLDivElement | null>(null);

  const currentUser = $derived($snapshot?.current_user);
  // Fail-closed: an unknown snapshot means no privileges, not full privileges.
  const securityLevel = $derived($snapshot?.security_level ?? 0);
  const isSuperAdmin = $derived(securityLevel >= 1000);
  // The backend blocks writes and user administration until a seeded account
  // replaces its default password, so the dialog is not dismissible.
  const mustChangePassword = $derived($snapshot?.password_change_required === true);

  async function handleLogout() {
    try {
      await api.logout();
      log("Wylogowano użytkownika", "info");
    } catch (e) {
      log(`Błąd wylogowania: ${e}`, "err");
    }
  }

  const validation = $derived(validateProject($project));

  const PANE_DEFAULTS = { left: 240, right: 280, bottom: 160 };
  const PANE_MIN = { left: 160, right: 200, bottom: 72 };
  const PANE_STORAGE_KEY = "proscada.pane.sizes";

  function loadPaneSizes() {
    try {
      const raw = localStorage.getItem(PANE_STORAGE_KEY);
      if (!raw) return { ...PANE_DEFAULTS };
      const parsed = JSON.parse(raw) as Partial<typeof PANE_DEFAULTS>;
      return {
        left: clamp(Number(parsed.left) || PANE_DEFAULTS.left, PANE_MIN.left, 560),
        right: clamp(Number(parsed.right) || PANE_DEFAULTS.right, PANE_MIN.right, 560),
        bottom: clamp(Number(parsed.bottom) || PANE_DEFAULTS.bottom, PANE_MIN.bottom, 520),
      };
    } catch {
      return { ...PANE_DEFAULTS };
    }
  }

  function clamp(n: number, min: number, max: number) {
    return Math.max(min, Math.min(max, n));
  }

  const initialPanes = loadPaneSizes();
  let paneLeft = $state(initialPanes.left);
  let paneRight = $state(initialPanes.right);
  let paneBottom = $state(initialPanes.bottom);
  let activeSplit = $state<"left" | "right" | "bottom" | null>(null);

  function persistPaneSizes() {
    localStorage.setItem(
      PANE_STORAGE_KEY,
      JSON.stringify({ left: paneLeft, right: paneRight, bottom: paneBottom }),
    );
  }

  function startSplit(which: "left" | "right" | "bottom", e: PointerEvent) {
    e.preventDefault();
    activeSplit = which;
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture(e.pointerId);
  }

  function onSplitMove(e: PointerEvent) {
    if (!activeSplit || !workspaceEl) return;
    const rect = workspaceEl.getBoundingClientRect();
    if (activeSplit === "left") {
      paneLeft = clamp(e.clientX - rect.left, PANE_MIN.left, Math.max(PANE_MIN.left, rect.width - paneRight - 280));
    } else if (activeSplit === "right") {
      paneRight = clamp(rect.right - e.clientX, PANE_MIN.right, Math.max(PANE_MIN.right, rect.width - paneLeft - 280));
    } else {
      const maxBottom = Math.max(PANE_MIN.bottom, rect.height - 120);
      paneBottom = clamp(rect.bottom - e.clientY, PANE_MIN.bottom, maxBottom);
    }
  }

  function endSplit(e: PointerEvent) {
    if (!activeSplit) return;
    activeSplit = null;
    try {
      (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
    } catch {
      /* already released */
    }
    persistPaneSizes();
  }

  function resetSplit(which: "left" | "right" | "bottom") {
    if (which === "left") paneLeft = PANE_DEFAULTS.left;
    else if (which === "right") paneRight = PANE_DEFAULTS.right;
    else paneBottom = PANE_DEFAULTS.bottom;
    persistPaneSizes();
  }

  /** Keyboard resizing so the panes are not mouse-only. */
  function nudgeSplit(which: "left" | "right" | "bottom", delta: number) {
    if (which === "left") paneLeft = clamp(paneLeft + delta, PANE_MIN.left, 560);
    else if (which === "right") paneRight = clamp(paneRight - delta, PANE_MIN.right, 560);
    else paneBottom = clamp(paneBottom - delta, PANE_MIN.bottom, 520);
    persistPaneSizes();
  }

  /** Native macOS/Windows title bar — no duplicate in-app title strip. */
  $effect(() => {
    const modeLabel = $mode === "runtime" ? "Runtime" : "Designer";
    const projectName = $project?.name?.trim();
    const dirtyMark = $dirty ? " •" : "";
    const parts = [
      "ProScada — Engineering Workstation",
      `v1.0 ${modeLabel}`,
    ];
    if (projectName) parts.push(projectName + dirtyMark);
    const title = parts.join(" · ");
    try {
      void getCurrentWindow()
        .setTitle(title)
        .catch(() => {
          document.title = title;
        });
    } catch {
      document.title = title;
    }
  });

  $effect(() => {
    if ($focusPropertiesTick > 0 && propertiesEl) {
      propertiesEl.scrollTop = 0;
      propertiesEl.classList.add("props-flash");
      setTimeout(() => propertiesEl?.classList.remove("props-flash"), 600);
    }
  });

  onMount(() => {
    initApp();
    const onKey = (e: KeyboardEvent) => {
      if ($mode !== "designer") {
        if ((e.ctrlKey || e.metaKey) && e.key === "s") {
          e.preventDefault();
          persistProject();
        }
        return;
      }
      const mod = e.ctrlKey || e.metaKey;
      const t = e.target as HTMLElement | null;
      const typing =
        t &&
        (t.tagName === "INPUT" ||
          t.tagName === "TEXTAREA" ||
          t.tagName === "SELECT" ||
          t.isContentEditable);
      if (typing && e.key !== "Escape") return;

      if (mod && e.key.toLowerCase() === "z") {
        e.preventDefault();
        if (e.shiftKey) redoAction();
        else undoAction();
        return;
      }
      if (mod && e.key.toLowerCase() === "y") {
        e.preventDefault();
        redoAction();
        return;
      }
      if (e.key === "Delete" || e.key === "Backspace") {
        if (!typing) {
          e.preventDefault();
          deleteSelectedWidget();
        }
        return;
      }
      if (mod && e.key.toLowerCase() === "s") {
        e.preventDefault();
        persistProject(e.shiftKey);
        return;
      }
      if (mod && e.key.toLowerCase() === "c") {
        e.preventDefault();
        copySelectedWidgets();
        return;
      }
      if (mod && e.key.toLowerCase() === "x") {
        e.preventDefault();
        cutSelectedWidgets();
        return;
      }
      if (mod && e.key.toLowerCase() === "v") {
        e.preventDefault();
        pasteWidgets();
        return;
      }
      if (mod && e.key.toLowerCase() === "d") {
        e.preventDefault();
        duplicateSelected();
        return;
      }
      if (mod && e.key.toLowerCase() === "a") {
        e.preventDefault();
        selectAllWidgets();
        return;
      }
      if (mod && e.shiftKey && e.key.toLowerCase() === "w") {
        e.preventDefault();
        startWindowOpen.update((v) => !v);
        return;
      }
      if (mod && e.shiftKey && e.key.toLowerCase() === "g") {
        e.preventDefault();
        ungroupSelectedWidgets();
        return;
      }
      if (mod && e.key.toLowerCase() === "g") {
        e.preventDefault();
        groupSelectedWidgets();
        return;
      }
      if (mod && e.key.toLowerCase() === "m") {
        e.preventDefault();
        multiCopySelected(3, 24, 24);
        return;
      }
      if (mod && e.key.toLowerCase() === "l") {
        e.preventDefault();
        toggleLockSelected();
        return;
      }
      if (e.key === "F4") {
        e.preventDefault();
        openAttributesPanel();
        leftTab = "solution";
        return;
      }
      if (
        e.key === "ArrowLeft" ||
        e.key === "ArrowRight" ||
        e.key === "ArrowUp" ||
        e.key === "ArrowDown"
      ) {
        if (!typing && ($selectedWidgetIds.length > 0 || $selectedWidgetId)) {
          e.preventDefault();
          const step = e.shiftKey ? 10 : 1;
          let dx = 0;
          let dy = 0;
          if (e.key === "ArrowLeft") dx = -step;
          if (e.key === "ArrowRight") dx = step;
          if (e.key === "ArrowUp") dy = -step;
          if (e.key === "ArrowDown") dy = step;
          moveSelectedWidgets(dx, dy);
          return;
        }
      }
    };
    window.addEventListener("keydown", onKey);
    const auditTimer = setInterval(() => refreshAudit(), 3000);
    return () => {
      window.removeEventListener("keydown", onKey);
      clearInterval(auditTimer);
    };
  });

  async function onWrite(tagId: string, value: number) {
    if ($project?.session_config?.pin_challenge_on_write) {
      pinChallengeAction = `Zapis wartości ${tagId} = ${value}`;
      pendingWriteFn = async () => {
        try {
          await api.writeTag(tagId, value);
          log(`Write ${tagId} = ${value}`, "ok");
          await refreshAudit();
        } catch (e) {
          log(`Write failed: ${e}`, "err");
          throw e;
        }
      };
      pinChallengeOpen = true;
      return;
    }

    try {
      await api.writeTag(tagId, value);
      log(`Write ${tagId} = ${value}`, "ok");
      await refreshAudit();
    } catch (e) {
      log(`Write failed: ${e}`, "err");
      throw e;
    }
  }

  async function reloadWaterTank() {
    try {
      const p = await api.loadBuiltinWaterTank();
      const normalized = ensureProjectTree(p);
      project.set(normalized);
      dirty.set(false);
      log("Reloaded factory Water Tank project", "ok");
    } catch (e) {
      log(`Reload failed: ${e}`, "err");
    }
  }

  onMount(() => {
    const onAlarmAction = (event: Event) => {
      const detail = (event as CustomEvent<{ action?: string; alarmId?: string }>).detail;
      if (detail?.action !== "ack" || !detail.alarmId) return;
      void api
        .ackAlarm(detail.alarmId)
        .then(() => {
          log(`Alarm ACK requested: ${detail.alarmId}`, "ok");
          return refreshAudit();
        })
        .catch((error: unknown) => log(`Alarm ACK failed: ${error}`, "err"));
    };
    const onNavigate = (event: Event) => {
      const detail = (event as CustomEvent<{ target?: string }>).detail;
      const target = detail?.target;
      if (!target) return;
      const key = target.startsWith("screen:") ? target.slice(7) : target.replace(/^\/+/, "");
      const form = $project?.forms.find(
        (candidate) => candidate.id === key || candidate.name.toLowerCase() === key.toLowerCase(),
      );
      if (!form) {
        log(`Navigation target not found: ${target}`, "warn");
        return;
      }
      selectedFormId.set(form.id);
      log(`Navigate → ${form.name}`, "info");
    };
    const onDialogAction = (event: Event) => {
      const detail = (event as CustomEvent<{ action?: string; sourceWidgetId?: string }>).detail;
      log(`Dialog ${detail?.action ?? "action"} · ${detail?.sourceWidgetId ?? "unknown"}`, "info");
    };
    const onOpenLogin = () => {
      loginModalOpen = true;
    };
    window.addEventListener("proscada:alarm-action", onAlarmAction);
    window.addEventListener("proscada:navigate", onNavigate);
    window.addEventListener("proscada:dialog-action", onDialogAction);
    window.addEventListener("proscada:open-login", onOpenLogin);
    return () => {
      window.removeEventListener("proscada:alarm-action", onAlarmAction);
      window.removeEventListener("proscada:navigate", onNavigate);
      window.removeEventListener("proscada:dialog-action", onDialogAction);
      window.removeEventListener("proscada:open-login", onOpenLogin);
    };
  });

  const statusClass = $derived(
    !$snapshot
      ? "offline"
      : $snapshot.connected
        ? ""
        : $snapshot.last_error
          ? "offline"
          : "degraded",
  );

  const centerDoc = $derived(
    $selectedSolutionNode && isDocKind($selectedSolutionNode.kind)
      ? $selectedSolutionNode
      : null,
  );
</script>

<SettingsModal open={settingsOpen} onClose={() => (settingsOpen = false)} />
<StartWindow />
<AddDeviceModal open={$addDeviceModalOpen} onClose={() => addDeviceModalOpen.set(false)} />
<AddAlarmModal open={$addAlarmModalOpen} onClose={() => addAlarmModalOpen.set(false)} />
<AddVariableModal open={$addVariableModalOpen} onClose={() => addVariableModalOpen.set(false)} />

<LoginModal
  open={loginModalOpen}
  onclose={() => (loginModalOpen = false)}
  onsuccess={(u) => log(`Zalogowano jako ${u.username} [L${u.security_level}]`, "ok")}
/>

<PinChallengeModal
  open={pinChallengeOpen}
  actionName={pinChallengeAction}
  onclose={() => {
    pinChallengeOpen = false;
    pendingWriteFn = null;
  }}
  onsuccess={() => {
    if (pendingWriteFn) {
      pendingWriteFn();
      pendingWriteFn = null;
    }
  }}
/>

<UserManagementModal
  open={userMgmtModalOpen}
  onclose={() => (userMgmtModalOpen = false)}
/>

<ChangePasswordModal
  open={mustChangePassword}
  mandatory={true}
  username={currentUser?.username ?? ""}
  onsuccess={() => log("Hasło zostało zmienione", "ok")}
/>

<div class="shell">
  <MenuBar
    {leftTab}
    onLeftTab={(t) => (leftTab = t)}
    onNewWaterTank={reloadWaterTank}
    onOpenSettings={() => (settingsOpen = true)}
  />

  <div class="toolbar">
    <button
      class="tb"
      class:primary={$mode === "designer"}
      title="Design"
      onclick={() => switchMode("designer")}
      disabled={$mode === "designer"}
    >
      Design
    </button>
    <button
      class="tb"
      class:primary={$mode === "runtime"}
      title="Start Runtime (F5)"
      onclick={() => switchMode("runtime")}
      disabled={$mode === "runtime"}
    >
      ▶ Start
    </button>
    <div class="sep"></div>
    <button class="tb" title="Save (Ctrl+S)" onclick={() => persistProject()}>Save</button>
    <button class="tb" title="Application Settings" onclick={() => (settingsOpen = true)}>⚙️ Settings</button>
    {#if $mode === "designer"}
      <button class="tb" title="Add New Screen" onclick={() => addNewForm()}>+ Screen</button>
    {/if}
    <div class="sep"></div>
    <button class="tb" title="Connect Modbus" onclick={() => connectDevice()}>Connect</button>
    <button class="tb" title="Stop Poll" onclick={() => disconnectDevice()}>Stop</button>
    <div class="sep"></div>

    <!-- Security & User Identity Badge Controls -->
    <div class="user-identity-badge" title={`Zalogowany użytkownik: ${currentUser?.display_name ?? 'Podgląd'}`}>
      <span class="user-icon">👤</span>
      <span class="username">{currentUser?.username ?? "Podgląd"}</span>
      <span class="level-pill" class:admin={securityLevel >= 1000} class:engineer={securityLevel >= 500 && securityLevel < 1000} class:operator={securityLevel >= 100 && securityLevel < 500}>
        L{securityLevel}
      </span>
    </div>

    <button class="tb user-btn" title="Logowanie / PIN HMI" onclick={() => (loginModalOpen = true)}>
      🔑 Logowanie
    </button>

    {#if isSuperAdmin}
      <button class="tb user-btn" title="Zarządzanie Użytkownikami i Uprawnieniami" onclick={() => (userMgmtModalOpen = true)}>
        👥 Użytkownicy
      </button>
    {/if}

    {#if currentUser}
      <button class="tb user-btn danger" title="Wyloguj sesję" onclick={handleLogout}>
        🚪
      </button>
    {/if}
    {#if $mode === "designer" && $selectedWidgetIds.length >= 2}
      <div class="sep"></div>
      <span class="align-label">Align:</span>
      <button class="tb align-btn" title="Align Left (do lewej)" onclick={() => alignSelectedWidgets("left")}>⇤ Left</button>
      <button class="tb align-btn" title="Align Center H (do środka poziom)" onclick={() => alignSelectedWidgets("center")}>↔ Center</button>
      <button class="tb align-btn" title="Align Right (do prawej)" onclick={() => alignSelectedWidgets("right")}>⇥ Right</button>
      <button class="tb align-btn" title="Align Top (do góry)" onclick={() => alignSelectedWidgets("top")}>⤒ Top</button>
      <button class="tb align-btn" title="Align Middle V (do środka pion)" onclick={() => alignSelectedWidgets("middle")}>↕ Middle</button>
      <button class="tb align-btn" title="Align Bottom (do dołu)" onclick={() => alignSelectedWidgets("bottom")}>⤓ Bottom</button>
    {/if}
    {#if $mode === "designer"}
      <div class="sep"></div>
      <button
        class="tb"
        class:primary={leftTab === "solution"}
        title="Solution Explorer"
        onclick={() => (leftTab = "solution")}>Solution</button
      >
      <button
        class="tb"
        class:primary={leftTab === "toolbox"}
        title="Toolbox"
        onclick={() => (leftTab = "toolbox")}>Toolbox</button
      >
      <button
        class="tb"
        class:primary={leftTab === "objects"}
        title="Document Outline"
        onclick={() => (leftTab = "objects")}>Outline</button
      >
      <button
        class="tb"
        class:primary={leftTab === "designSystem"}
        title="Project Design System"
        onclick={() => (leftTab = "designSystem")}>Styles</button
      >
      <button
        class="tb"
        class:primary={leftTab === "components"}
        title="Reusable Component Library"
        onclick={() => (leftTab = "components")}>Components</button
      >
      <button
        class="tb"
        class:primary={leftTab === "alarms"}
        title="Central Alarm Manager"
        onclick={() => (leftTab = "alarms")}>Alarms</button
      >
    {/if}
    <span class="tb-status">
      {$snapshot?.connected ? "ONLINE" : "OFFLINE"}
      · polls {$snapshot?.poll_count ?? 0}
      · {$snapshot?.last_poll_ms ?? 0} ms
      {#if $snapshot?.last_error}
        · {$snapshot.last_error}
      {/if}
    </span>
  </div>

  <div
    class="workspace"
    class:runtime-only={$mode === "runtime"}
    bind:this={workspaceEl}
    style:--pane-left="{paneLeft}px"
    style:--pane-right="{paneRight}px"
    style:--pane-bottom="{paneBottom}px"
  >
    {#if $project}
      {#if $mode === "designer"}
        <div class="solution">
          {#if leftTab === "solution"}
            <SolutionExplorer project={$project} design={true} />
          {:else if leftTab === "toolbox"}
            <Toolbox />
          {:else if leftTab === "designSystem"}
            <DesignSystemManager />
          {:else if leftTab === "components"}
            <ComponentLibraryManager />
          {:else if leftTab === "alarms"}
            <AlarmManagerEditor />
          {:else if leftTab === "objects" && $activeForm}
            <ObjectList form={$activeForm} />
          {/if}
        </div>
      {:else}
        <div class="solution">
          <SolutionExplorer project={$project} design={false} />
        </div>
      {/if}

      <button
        type="button"
        class="splitter splitter-v split-v-left"
        class:active={activeSplit === "left"}
        aria-label="Szerokość panelu lewego"
        title="Drag to resize · double-click to reset · arrows to nudge"
        onpointerdown={(e) => startSplit("left", e)}
        onpointermove={onSplitMove}
        onpointerup={endSplit}
        onpointercancel={endSplit}
        ondblclick={() => resetSplit("left")}
        onkeydown={resizeOnKey("vertical", (delta) => nudgeSplit("left", delta))}
      ></button>

      <div class="center">
        <!-- Interactive Unified Multi-Tabstrip (Screens + Active Document/Variables) -->
        <div class="tabstrip" style:display="flex" style:align-items="center">
          {#each $project.forms as f}
            <div
              class="tab"
              class:active={!centerDoc && $selectedFormId === f.id}
              role="button"
              tabindex="0"
              onclick={() => {
                selectSolutionNode(null);
                selectedFormId.set(f.id);
                selectedWidgetId.set(null);
              }}
              onkeydown={activate(() => {
                selectSolutionNode(null);
                selectedFormId.set(f.id);
                selectedWidgetId.set(null);
              })}
            >
              <span>{f.name}.form</span>
              {#if $mode === "designer" && $project.forms.length > 1 && !isMainScreen(f)}
                <button
                  type="button"
                  class="tab-close"
                  title="Usuń ekran {f.name}"
                  aria-label="Usuń ekran {f.name}"
                  onclick={(e) => {
                    e.stopPropagation();
                    deleteForm(f.id);
                  }}
                >
                  ✕
                </button>
              {/if}
            </div>
          {/each}

          {#if centerDoc}
            <!-- Active Document Tab (Variables / Script / Note) -->
            <div class="tab active">
              <span>{iconFor(centerDoc.kind)} {centerDoc.name}</span>
              <button
                type="button"
                class="tab-close"
                title="Zamknij zakładkę {centerDoc.name}"
                aria-label="Zamknij zakładkę {centerDoc.name}"
                onclick={(e) => {
                  e.stopPropagation();
                  selectSolutionNode(null);
                }}
              >
                ✕
              </button>
            </div>
          {/if}

          {#if $mode === "designer"}
            <button
              class="btn-new-tab"
              title="Add New Screen"
              onclick={() => addNewForm()}
            >
              +
            </button>
          {/if}
        </div>

        {#if centerDoc}
          <div class="doc-host">
            {#if centerDoc.kind === "variables"}
              <VariablesEditor scada={$project} design={$mode === "designer"} />
            {:else}
              <DocumentEditor node={centerDoc} design={$mode === "designer"} />
            {/if}
          </div>
        {:else}
          {#if $activeForm}
            <!-- Design and Run: always the currently selected screen (tab). -->
            <DesignerCanvas
              form={$activeForm}
              tagMap={$tagMap}
              design={$mode === "designer"}
              {onWrite}
            />
          {:else}
            <div
              class="canvas-wrap"
              style:display="flex"
              style:align-items="center"
              style:justify-content="center"
            >
              No form in project
            </div>
          {/if}
        {/if}
      </div>

      {#if $mode === "designer"}
        <button
          type="button"
          class="splitter splitter-v split-v-right"
          class:active={activeSplit === "right"}
          aria-label="Szerokość panelu właściwości"
          title="Drag to resize · double-click to reset · arrows to nudge"
          onpointerdown={(e) => startSplit("right", e)}
          onpointermove={onSplitMove}
          onpointerup={endSplit}
          onpointercancel={endSplit}
          ondblclick={() => resetSplit("right")}
          onkeydown={resizeOnKey("vertical", (delta) => nudgeSplit("right", delta))}
        ></button>
        <div class="properties" bind:this={propertiesEl}>
          <Properties
            widget={$selectedWidget}
            form={$activeForm}
            tags={$project.tags}
          />
        </div>
      {/if}

      <button
        type="button"
        class="splitter splitter-h split-h-bottom"
        class:active={activeSplit === "bottom"}
        aria-label="Wysokość panelu wyjściowego"
        title="Drag to resize Output / Alarms · double-click to reset · arrows to nudge"
        onpointerdown={(e) => startSplit("bottom", e)}
        onpointermove={onSplitMove}
        onpointerup={endSplit}
        onpointercancel={endSplit}
        ondblclick={() => resetSplit("bottom")}
        onkeydown={resizeOnKey("horizontal", (delta) => nudgeSplit("bottom", delta))}
      ></button>

      <div class="output">
        <OutputPanel snapshot={$snapshot} audit={$audit} />
      </div>
    {:else}
      <div style:grid-column="1 / -1" style:padding="40px">Loading project…</div>
    {/if}
  </div>

  <div class="statusbar {statusClass}">
    <span class="item">ProScada Engineering Workstation</span>
    <span class="item">
      {$snapshot?.connected ? "● Modbus master connected" : "○ Disconnected"}
    </span>
    <span class="item">Role: {$snapshot?.role ?? "viewer"}</span>
    {#if $mode === "designer" && $selectedWidgetIds.length >= 2}
      <span class="item align-item">
        <span class="sb-align-lbl">Align ({$selectedWidgetIds.length}):</span>
        <button class="sb-btn" title="Align Left (do lewej)" onclick={() => alignSelectedWidgets("left")}>⇤</button>
        <button class="sb-btn" title="Align Center H (do środka w poziomie)" onclick={() => alignSelectedWidgets("center")}>↔</button>
        <button class="sb-btn" title="Align Right (do prawej)" onclick={() => alignSelectedWidgets("right")}>⇥</button>
        <button class="sb-btn" title="Align Top (do góry)" onclick={() => alignSelectedWidgets("top")}>⤒</button>
        <button class="sb-btn" title="Align Middle V (do środka w pionie)" onclick={() => alignSelectedWidgets("middle")}>↕</button>
        <button class="sb-btn" title="Align Bottom (do dołu)" onclick={() => alignSelectedWidgets("bottom")}>⤓</button>
      </span>
    {/if}
    <span class="item">Mode: {$mode}</span>
    <span class="sb-copilot-pill" title="GitHub Copilot Engine active for ProScada Workstation">
      <span class="copilot-spark">✨</span> Copilot: Active
    </span>
    <button
      type="button"
      class="item sb-autosave-item"
      title="AutoSave Status · Click to open Application Settings"
      onclick={() => (settingsOpen = true)}
    >
      {#if !$appSettings.autosaveEnabled}
        <span class="sb-badge off">💾 AutoSave: OFF</span>
      {:else if !validation.valid}
        <span class="sb-badge warn">💾 AutoSave: ⚠️ {validation.errors.length} err (Skipped)</span>
      {:else if $appSettings.lastAutosaveStatus === 'ok'}
        <span class="sb-badge ok">💾 AutoSave: ON ({$appSettings.autosaveIntervalMinutes}m) · OK</span>
      {:else}
        <span class="sb-badge ok">💾 AutoSave: ON ({$appSettings.autosaveIntervalMinutes}m)</span>
      {/if}
    </button>
    <span class="item" style:margin-left="auto">
      IEC 62443 / ISA-18.2 practices · Lab use only · Not certified
    </span>
  </div>
</div>

<style>
  .sb-autosave-item {
    cursor: pointer;
    background: none;
    border: none;
    color: inherit;
    font: inherit;
    padding: 0;
  }
  .sb-autosave-item:focus-visible {
    outline: 2px solid #1f6feb;
    outline-offset: 1px;
  }
  .sb-copilot-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 1px 8px;
    border-radius: 10px;
    background: rgba(137, 87, 229, 0.2);
    border: 1px solid rgba(163, 113, 247, 0.45);
    color: #d2a8ff;
    font-weight: 700;
    font-size: 10.5px;
    letter-spacing: 0.02em;
    box-shadow: 0 0 8px rgba(163, 113, 247, 0.25);
  }
  .sb-badge {
    padding: 1px 6px;
    border-radius: 3px;
    font-weight: 700;
    font-size: 10.5px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.25);
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .sb-badge.ok {
    background: rgba(22, 163, 74, 0.35);
    border-color: #4ade80;
    color: #ffffff;
  }
  .sb-badge.warn {
    background: rgba(234, 179, 8, 0.35);
    border-color: #facc15;
    color: #ffffff;
  }
  .sb-badge.off {
    opacity: 0.75;
  }
  .tab-close {
    margin-left: 6px;
    font-size: 11px;
    font-weight: 800;
    color: #ef4444;
    cursor: pointer;
    border-radius: 50%;
    padding: 0 4px;
    opacity: 0.7;
  }
  .tab-close:hover {
    opacity: 1;
    background: rgba(239, 68, 68, 0.2);
  }
  .btn-new-tab {
    background: var(--vs-panel-header-bg, #2d2d2d);
    color: var(--vs-text, #cccccc);
    border: 1px dashed var(--vs-border, #444444);
    border-radius: 4px;
    font-size: 11px;
    font-weight: 700;
    padding: 3px 8px;
    margin-left: 8px;
    cursor: pointer;
  }
  .btn-new-tab:hover {
    background: var(--vs-hover, #3e3e42);
    color: #ffffff;
  }
  .user-identity-badge {
    display: flex;
    align-items: center;
    gap: 6px;
    background: #090d16;
    border: 1px solid #1e293b;
    padding: 3px 10px;
    border-radius: 6px;
    font-size: 11px;
    font-weight: 600;
    color: #f8fafc;
  }
  .level-pill {
    padding: 1px 5px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 800;
    background: #334155;
    color: #94a3b8;
  }
  .level-pill.admin { background: rgba(225, 29, 72, 0.25); color: #f43f5e; border: 1px solid rgba(225, 29, 72, 0.4); }
  .level-pill.engineer { background: rgba(168, 85, 247, 0.25); color: #c084fc; border: 1px solid rgba(168, 85, 247, 0.4); }
  .level-pill.operator { background: rgba(14, 165, 233, 0.25); color: #38bdf8; border: 1px solid rgba(14, 165, 233, 0.4); }
  .user-btn {
    font-weight: 600 !important;
  }
  .user-btn.danger:hover {
    background: #be123c !important;
    color: #ffffff !important;
  }
  .doc-host {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
</style>
