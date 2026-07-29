<script lang="ts">
  import {
    recentProjects,
    togglePinRecentProject,
    removeRecentProject,
    clearRecentProjects,
    type RecentProjectItem,
  } from "$lib/stores/recentProjects";
  import {
    project,
    newBlankProject,
    openRecentProjectItem,
    applyLoadedProject,
    importProjectFile,
    importProjectFromJson,
    startWindowOpen,
    log,
    snapshot,
    dirty,
    refreshSnapshotNow,
  } from "$lib/stores/app";
  import { api } from "$lib/services/api";
  import { errorMessage } from "$lib/utils/errors";
  import { activate } from "$lib/utils/a11y";
  import { createAndSaveNewProject } from "$lib/stores/projectStorage";
  import { ensureProjectTree } from "$lib/utils/projectTree";
  import { appSettings, updateAppSettings } from "$lib/stores/settings";

  let searchQuery = $state("");
  let searchInputEl = $state<HTMLInputElement | null>(null);
  let createModalOpen = $state(false);
  let confirmClearOpen = $state(false);
  let isDraggingOver = $state(false);
  let selectedIndex = $state(0);

  let newProjectName = $state("Nowy Projekt SCADA");
  let newProjectDesc = $state("");
  let selectedTemplate = $state<"blank" | "water_tank">("blank");
  const canEngineer = $derived(($snapshot?.security_level ?? 0) >= 500);

  async function handleQuickDevLogin() {
    try {
      await api.devLoginAdmin();
      await refreshSnapshotNow();
      log("Szybkie logowanie deweloperskie (admin) powiodło się", "ok");
    } catch (e) {
      log(`Błąd szybkiego logowania: ${errorMessage(e, "Błąd autoryzacji")}`, "err");
    }
  }

  async function handleLogoutFromStart() {
    try {
      await api.logout();
      await api.setMode("runtime");
      await refreshSnapshotNow();
      log("Wylogowano z poziomu okna startowego", "info");
    } catch (e) {
      log(`Błąd wylogowania: ${e}`, "err");
    }
  }

  // Sorted and filtered recent projects
  const filteredProjects = $derived.by(() => {
    const query = searchQuery.trim().toLowerCase();
    const list = $recentProjects.filter((p) => {
      if (!query) return true;
      return (
        p.name.toLowerCase().includes(query) ||
        (p.description && p.description.toLowerCase().includes(query)) ||
        (p.path && p.path.toLowerCase().includes(query)) ||
        p.id.toLowerCase().includes(query)
      );
    });

    // Pinned first, then chronological lastOpened desc
    return list.sort((a, b) => {
      if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
      return new Date(b.lastOpened).getTime() - new Date(a.lastOpened).getTime();
    });
  });

  // Keep selected index within bounds
  $effect(() => {
    if (selectedIndex >= filteredProjects.length) {
      selectedIndex = Math.max(0, filteredProjects.length - 1);
    }
  });

  // Focus search input automatically when start window opens
  $effect(() => {
    if ($startWindowOpen && searchInputEl) {
      setTimeout(() => searchInputEl?.focus(), 50);
    }
  });

  function closeWindow() {
    startWindowOpen.set(false);
  }

  function formatDate(isoString: string): string {
    try {
      const d = new Date(isoString);
      if (isNaN(d.getTime())) return "n/a";
      const now = new Date();
      const diffMs = now.getTime() - d.getTime();
      const diffMins = Math.floor(diffMs / 60000);
      const diffHours = Math.floor(diffMins / 24);
      const diffDays = Math.floor(diffHours / 24);

      if (diffMins < 2) return "Przed chwilą";
      if (diffMins < 60) return `${diffMins} min temu`;
      if (diffHours < 24 && d.getDate() === now.getDate()) {
        return `Dzisiaj, ${d.toLocaleTimeString("pl-PL", { hour: "2-digit", minute: "2-digit" })}`;
      }
      if (diffDays === 1) {
        return `Wczoraj, ${d.toLocaleTimeString("pl-PL", { hour: "2-digit", minute: "2-digit" })}`;
      }
      return d.toLocaleDateString("pl-PL", {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return isoString;
    }
  }

  async function handleSelectRecent(item: RecentProjectItem) {
    if (!item || !canEngineer) return;
    try {
      if ($project && $project.id === item.id) {
        log(`Otwarto projekt z historii: ${item.name}`, "info");
        closeWindow();
        return;
      }
      const ok = await openRecentProjectItem(item);
      if (ok) closeWindow();
    } catch (e) {
      log(`Nie udało się otworzyć projektu z historii: ${e}`, "err");
    }
  }

  async function handleCreateSubmit() {
    if (!canEngineer || !newProjectName.trim()) return;
    try {
      if (selectedTemplate === "water_tank") {
        const p = await api.getBuiltinWaterTank();
        const customP = { ...ensureProjectTree(p), name: newProjectName.trim(), description: newProjectDesc.trim() };
        const result = await createAndSaveNewProject(newProjectName.trim(), newProjectDesc.trim(), customP);
        if (result) {
          applyLoadedProject(result.project, `Utworzono projekt na bazie szablonu Water Tank: ${newProjectName}`, result.path);
          dirty.set(result.path === null);
          if (result.path === null) {
            log("Nowy projekt jest aktywny w engine, ale nie został zapisany na dysku", "warn");
          }
        }
      } else {
        await newBlankProject(newProjectName.trim(), newProjectDesc.trim());
      }
      createModalOpen = false;
      closeWindow();
    } catch (e) {
      log(`Błąd podczas tworzenia projektu: ${e}`, "err");
    }
  }

  async function handleOpenProject() {
    if (!canEngineer) return;
    await importProjectFile();
    closeWindow();
  }

  async function handleImportProject() {
    if (!canEngineer) return;
    await importProjectFile();
    closeWindow();
  }

  async function handleLoadWaterTankDemo() {
    if (!canEngineer) return;
    try {
      const p = await api.getBuiltinWaterTank();
      const saved = await api.saveProject(p);
      applyLoadedProject(
        ensureProjectTree(saved),
        "Załadowano projekt demonstracyjny Stacji Pomp Wodnych",
        null,
      );
      dirty.set(true);
      log("Załadowano projekt demonstracyjny Stacji Pomp Wodnych", "ok");
      closeWindow();
    } catch (e) {
      log(`Błąd ładowania demo Water Tank: ${e}`, "err");
    }
  }

  function onKeyDown(e: KeyboardEvent) {
    if (!$startWindowOpen) return;
    if (e.key === "Escape") {
      if (createModalOpen) {
        createModalOpen = false;
        return;
      }
      if (confirmClearOpen) {
        confirmClearOpen = false;
        return;
      }
      closeWindow();
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (filteredProjects.length > 0) {
        selectedIndex = (selectedIndex + 1) % filteredProjects.length;
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (filteredProjects.length > 0) {
        selectedIndex = (selectedIndex - 1 + filteredProjects.length) % filteredProjects.length;
      }
    } else if (e.key === "Enter" && !createModalOpen && !confirmClearOpen) {
      const item = filteredProjects[selectedIndex];
      if (item) {
        e.preventDefault();
        void handleSelectRecent(item);
      }
    }
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDraggingOver = false;
    if (!canEngineer) {
      log("Import projektu wymaga roli Engineer lub Administrator", "warn");
      return;
    }
    const file = e.dataTransfer?.files?.[0];
    if (!file) return;
    if (file.name.endsWith(".json") || file.name.endsWith(".proscada.json")) {
      try {
        const text = await file.text();
        await importProjectFromJson(text);
        log(`Zaimportowano przeciągnięty plik: ${file.name}`, "ok");
        closeWindow();
      } catch (err) {
        log(`Nieprawidłowy plik projektu: ${err}`, "err");
      }
    } else {
      log("Upuszczony plik musi mieć rozszerzenie .json lub .proscada.json", "warn");
    }
  }
</script>

<svelte:window onkeydown={onKeyDown} />

{#if $startWindowOpen}
  <div
    class="vs-start-overlay"
    class:drag-active={isDraggingOver}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    aria-labelledby="start-window-title"
    ondragover={(e) => {
      e.preventDefault();
      isDraggingOver = true;
    }}
    ondragleave={(e) => {
      e.preventDefault();
      isDraggingOver = false;
    }}
    ondrop={handleDrop}
  >
    <div class="vs-start-modal">
      <!-- Top Bar Header -->
      <div class="vs-start-header">
        <div class="vs-start-brand">
          <div class="vs-logo-icon">SCADA</div>
          <div class="vs-brand-text">
            <span id="start-window-title" class="vs-brand-title">ProScada Engineering Workstation</span>
            <span class="vs-brand-sub">Visual Studio–Style Project Launcher · Engineering Workstation</span>
          </div>
        </div>
        <div class="vs-header-right">
          <div class="vs-user-badge" class:logged-in={canEngineer}>
            {#if canEngineer}
              <span class="vs-user-status">👤 {$snapshot?.current_user?.display_name || $snapshot?.actor || "Użytkownik"} ({$snapshot?.role})</span>
            {:else}
              <span class="vs-user-status dim">🟡 Gość (Brak uprawnień edycji)</span>
            {/if}
          </div>
          <button class="vs-start-close-btn" title="Zamknij ekran startowy (Esc)" onclick={closeWindow}>
            ✕
          </button>
        </div>
      </div>

      <!-- Drag Overlay Banner -->
      {#if isDraggingOver}
        <div class="vs-drag-banner">
          <div class="vs-drag-icon">📦</div>
          <div class="vs-drag-text">Upuść plik projektu (.proscada.json), aby go natychmiast otworzyć!</div>
        </div>
      {/if}

      <!-- Main Body Container -->
      <div class="vs-start-body">
        <!-- Left Column: Recent Projects History -->
        <div class="vs-start-col-left">
          <div class="vs-col-title">
            <span>Niedawno otwarte ({filteredProjects.length})</span>
            {#if $recentProjects.length > 0}
              <button
                class="vs-clear-history-btn"
                title="Wyczyść całą historię"
                onclick={() => (confirmClearOpen = true)}
              >
                Wyczyść historię
              </button>
            {/if}
          </div>

          <div class="vs-search-box">
            <span class="vs-search-icon">🔍</span>
            <input
              bind:this={searchInputEl}
              type="text"
              class="vs-search-input"
              placeholder="Szukaj w niedawno otwartych (nawiguj strzałkami ⬆️/⬇️ + Enter)..."
              bind:value={searchQuery}
            />
            {#if searchQuery}
              <button class="vs-search-clear" onclick={() => (searchQuery = "")}>✕</button>
            {/if}
          </div>

          <div class="vs-recent-list" role="listbox" aria-label="Lista niedawno otwartych projektów">
            {#if filteredProjects.length === 0}
              <div class="vs-recent-empty">
                {#if searchQuery}
                  Brak projektów pasujących do frazy „{searchQuery}”.
                {:else}
                  Brak niedawno otwartych projektów. Rozpocznij od utworzenia nowego projektu lub załadowania szablonu!
                {/if}
              </div>
            {:else}
              {#each filteredProjects as p, index (p.id)}
                <div
                  class="vs-recent-card"
                  class:selected={index === selectedIndex}
                  role="option"
                  aria-selected={index === selectedIndex}
                  tabindex="0"
                  onclick={() => handleSelectRecent(p)}
                  onkeydown={activate(() => handleSelectRecent(p))}
                  onmouseenter={() => (selectedIndex = index)}
                >
                  <div class="vs-recent-icon" class:pinned={p.pinned}>
                    {p.pinned ? "📌" : "📄"}
                  </div>
                  <div class="vs-recent-info">
                    <div class="vs-recent-name">{p.name}</div>
                    <div class="vs-recent-meta">
                      {#if p.description}
                        <span class="vs-recent-desc">{p.description}</span>
                      {:else if p.path}
                        <span class="vs-recent-path">{p.path}</span>
                      {:else}
                        <span class="vs-recent-id">ID: {p.id}</span>
                      {/if}
                    </div>
                  </div>
                  <div class="vs-recent-time">
                    {formatDate(p.lastOpened)}
                  </div>
                  <div class="vs-recent-actions">
                    <button
                      type="button"
                      class="vs-action-icon"
                      class:active={p.pinned}
                      title={p.pinned ? "Odpnij projekt z góry listy" : "Przypnij projekt na górze listy"}
                      onclick={(e) => {
                        e.stopPropagation();
                        togglePinRecentProject(p.id);
                      }}
                    >
                      📌
                    </button>
                    <button
                      type="button"
                      class="vs-action-icon remove"
                      title="Usuń z listy historii"
                      onclick={(e) => {
                        e.stopPropagation();
                        removeRecentProject(p.id);
                      }}
                    >
                      ✕
                    </button>
                  </div>
                </div>
              {/each}
            {/if}
          </div>
        </div>

        <!-- Right Column: Quick Action Cards -->
        <div class="vs-start-col-right">
          <div class="vs-col-title">Rozpocznij pracę</div>

          <div class="vs-action-cards">
            <!-- Action 1: Create New Project -->
            <button type="button" class="vs-action-card primary" disabled={!canEngineer} onclick={() => (createModalOpen = true)}>
              <div class="vs-card-icon">➕</div>
              <div class="vs-card-content">
                <div class="vs-card-title">Stwórz nowy projekt</div>
                <div class="vs-card-sub">
                  Rozpocznij od czystego szablonu synoptyki SCADA lub gotowego wzorca stacji pomp.
                </div>
              </div>
              <div class="vs-card-arrow">➔</div>
            </button>

            <!-- Action 2: Open Existing Project -->
            <button type="button" class="vs-action-card" disabled={!canEngineer} onclick={handleOpenProject}>
              <div class="vs-card-icon">📂</div>
              <div class="vs-card-content">
                <div class="vs-card-title">Otwórz plik projektu</div>
                <div class="vs-card-sub">
                  Przeglądaj pliki konfiguracji SCADA (.proscada.json lub .json) z dysku.
                </div>
              </div>
              <div class="vs-card-arrow">➔</div>
            </button>

            <!-- Action 3: Import Project Package -->
            <button type="button" class="vs-action-card" disabled={!canEngineer} onclick={handleImportProject}>
              <div class="vs-card-icon">📦</div>
              <div class="vs-card-content">
                <div class="vs-card-title">Importuj projekt</div>
                <div class="vs-card-sub">
                  Zaimportuj zewnętrzną paczkę lub plik JSON z pełną walidacją schematu.
                </div>
              </div>
              <div class="vs-card-arrow">➔</div>
            </button>

            <!-- Action 4: Load Built-in Demo -->
            <button type="button" class="vs-action-card accent" disabled={!canEngineer} onclick={handleLoadWaterTankDemo}>
              <div class="vs-card-icon">🏭</div>
              <div class="vs-card-content">
                <div class="vs-card-title">Stacja Pomp Wodnych (Demo)</div>
                <div class="vs-card-sub">
                  Wbudowany projekt demonstracyjny IEC 62443 / ISA-18.2 z obsługą Modbus TCP.
                </div>
              </div>
              <div class="vs-card-arrow">➔</div>
            </button>
          </div>
          {#if !canEngineer}
            <div class="vs-auth-card">
              <div class="vs-auth-icon">🔒</div>
              <div class="vs-auth-info">
                <div class="vs-auth-title">Brak uprawnień edytora</div>
                <div class="vs-auth-desc">
                  Tworzenie, edycja i otwieranie projektów wymaga logowania jako <strong>Engineer</strong> lub <strong>Administrator</strong>.
                </div>
              </div>
              <div class="vs-auth-actions">
                <button
                  type="button"
                  class="vs-btn-pri"
                  onclick={() => window.dispatchEvent(new CustomEvent("proscada:open-login"))}
                >
                  🔑 Zaloguj się
                </button>
                {#if import.meta.env.DEV}
                  <button
                    type="button"
                    class="vs-btn-dev"
                    title="Tryb deweloperski: zaloguj od razu jako admin (admin/admin123)"
                    onclick={handleQuickDevLogin}
                  >
                    ⚡ Szybkie logowanie (Admin)
                  </button>
                {/if}
              </div>
            </div>
          {:else}
            <div class="vs-auth-card logged-in">
              <div class="vs-auth-icon">✅</div>
              <div class="vs-auth-info">
                <div class="vs-auth-title">Zalogowano: {$snapshot?.current_user?.display_name || $snapshot?.actor}</div>
                <div class="vs-auth-desc">Rola: <strong>{$snapshot?.role}</strong> (Security: {$snapshot?.security_level})</div>
              </div>
              <button type="button" class="vs-btn-sec" onclick={handleLogoutFromStart}>
                Wyloguj
              </button>
            </div>
          {/if}
        </div>
      </div>

      <!-- Footer Bar -->
      <div class="vs-start-footer">
        <label class="vs-show-start-toggle">
          <input
            type="checkbox"
            checked={$appSettings.showStartWindowOnStart !== false}
            onchange={(e) =>
              updateAppSettings({ showStartWindowOnStart: e.currentTarget.checked })}
          />
          <span>Pokazuj ten ekran przy każdym uruchamianiu aplikacji</span>
        </label>
        <button class="vs-footer-btn" onclick={closeWindow}>
          Kontynuuj bez zmian (Otwórz obszar roboczy) ➔
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Sub-Modal: Create New Project -->
{#if createModalOpen}
  <div class="vs-submodal-overlay" role="dialog" aria-modal="true">
    <div class="vs-submodal">
      <div class="vs-submodal-header">
        <h3>➕ Tworzenie nowego projektu SCADA</h3>
        <button class="vs-start-close-btn" onclick={() => (createModalOpen = false)}>✕</button>
      </div>

      <div class="vs-submodal-body">
        <div class="vs-field">
          <label for="new-project-name">Nazwa projektu</label>
          <input
            id="new-project-name"
            type="text"
            class="vs-input"
            bind:value={newProjectName}
            placeholder="np. Stacja Pomp Czystej Wody #1"
          />
        </div>

        <div class="vs-field">
          <label for="new-project-desc">Opis projektu (opcjonalny)</label>
          <input
            id="new-project-desc"
            type="text"
            class="vs-input"
            bind:value={newProjectDesc}
            placeholder="np. Główna pompownia z 2 agregatami i regulacją poziomu"
          />
        </div>

        <div class="vs-field">
          <span class="vs-field-label">Wybór szablonu początkowego</span>
          <div class="vs-template-options">
            <div
              class="vs-template-card"
              class:selected={selectedTemplate === "blank"}
              role="radio"
              aria-checked={selectedTemplate === "blank"}
              tabindex="0"
              onclick={() => (selectedTemplate = "blank")}
              onkeydown={activate(() => (selectedTemplate = "blank"))}
            >
              <div class="vs-tmpl-radio">{selectedTemplate === "blank" ? "●" : "○"}</div>
              <div class="vs-tmpl-details">
                <div class="vs-tmpl-title">Czysty projekt SCADA</div>
                <div class="vs-tmpl-desc">Pusty ekran startowy ze skrajną elastycznością konfiguracyjną.</div>
              </div>
            </div>

            <div
              class="vs-template-card"
              class:selected={selectedTemplate === "water_tank"}
              role="radio"
              aria-checked={selectedTemplate === "water_tank"}
              tabindex="0"
              onclick={() => (selectedTemplate = "water_tank")}
              onkeydown={activate(() => (selectedTemplate = "water_tank"))}
            >
              <div class="vs-tmpl-radio">{selectedTemplate === "water_tank" ? "●" : "○"}</div>
              <div class="vs-tmpl-details">
                <div class="vs-tmpl-title">Stacja Pomp Dual-Pump (Water Tank Demo)</div>
                <div class="vs-tmpl-desc">Gotowy szablon synoptyki z tagami Modbus, wykresami i alarmami.</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="vs-submodal-footer">
        <button class="vs-btn-sec" onclick={() => (createModalOpen = false)}>Anuluj</button>
        <button class="vs-btn-pri" onclick={handleCreateSubmit}>Utwórz projekt</button>
      </div>
    </div>
  </div>
{/if}

<!-- Sub-Modal: Confirm Clear History -->
{#if confirmClearOpen}
  <div class="vs-submodal-overlay" role="dialog" aria-modal="true">
    <div class="vs-submodal vs-submodal-sm">
      <div class="vs-submodal-header">
        <h3>⚠️ Czyszczenie historii projektów</h3>
        <button class="vs-start-close-btn" onclick={() => (confirmClearOpen = false)}>✕</button>
      </div>

      <div class="vs-submodal-body">
        <p style="margin:0; font-size:13px; color: var(--vs-text, #cccccc); line-height: 1.5;">
          Czy na pewno chcesz usunąć wszystkie projekty z listy historii? Sama zawartość plików projektów na dysku pozostanie nietknięta.
        </p>
      </div>

      <div class="vs-submodal-footer">
        <button class="vs-btn-sec" onclick={() => (confirmClearOpen = false)}>Anuluj</button>
        <button
          class="vs-btn-danger"
          onclick={() => {
            clearRecentProjects();
            confirmClearOpen = false;
          }}
        >
          Wyczyść historię
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .vs-start-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 9000;
    background: rgba(18, 18, 18, 0.85);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    animation: fadeIn 0.15s ease-out;
    transition: background 0.15s ease;
  }

  .vs-start-overlay.drag-active {
    background: rgba(0, 122, 204, 0.35);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: scale(0.99);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .vs-start-modal {
    position: relative;
    width: 100%;
    max-width: 1140px;
    height: 720px;
    max-height: 90vh;
    background: var(--vs-bg, #1e1e1e);
    border: 1px solid var(--vs-border, #3b3b3b);
    border-radius: 8px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .vs-drag-banner {
    position: absolute;
    top: 70px;
    left: 20px;
    right: 20px;
    z-index: 9200;
    padding: 20px;
    background: #007acc;
    color: #ffffff;
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    font-size: 16px;
    font-weight: 700;
    animation: bounce 0.3s ease;
  }

  .vs-drag-icon {
    font-size: 32px;
  }

  .vs-start-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    background: var(--vs-panel-header-bg, #252526);
    border-bottom: 1px solid var(--vs-border, #333333);
  }

  .vs-header-right {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .vs-user-badge {
    padding: 4px 10px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--vs-border, #3c3c3c);
    border-radius: 14px;
    font-size: 11.5px;
    color: var(--vs-text-dim, #aaaaaa);
  }

  .vs-user-badge.logged-in {
    background: rgba(16, 185, 129, 0.12);
    border-color: rgba(16, 185, 129, 0.4);
    color: #10b981;
    font-weight: 600;
  }

  .vs-user-status.dim {
    color: var(--vs-text-dim, #aaaaaa);
  }

  .vs-auth-card {
    margin-top: 16px;
    padding: 16px;
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.25);
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .vs-auth-card.logged-in {
    background: rgba(16, 185, 129, 0.08);
    border-color: rgba(16, 185, 129, 0.25);
    flex-direction: row;
    align-items: center;
  }

  .vs-auth-icon {
    font-size: 22px;
  }

  .vs-auth-info {
    flex: 1;
  }

  .vs-auth-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--vs-text-bright, #ffffff);
    margin-bottom: 3px;
  }

  .vs-auth-desc {
    font-size: 11.5px;
    color: var(--vs-text-dim, #aaaaaa);
    line-height: 1.4;
  }

  .vs-auth-actions {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
  }

  .vs-btn-dev {
    background: linear-gradient(135deg, #8b5cf6, #6366f1);
    border: none;
    color: #ffffff;
    padding: 6px 14px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: filter 0.15s ease;
  }

  .vs-btn-dev:hover {
    filter: brightness(1.15);
  }

  .vs-start-brand {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .vs-logo-icon {
    width: 38px;
    height: 38px;
    background: linear-gradient(135deg, #007acc, #005999);
    color: #ffffff;
    font-weight: 900;
    font-size: 11px;
    letter-spacing: 0.5px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 8px rgba(0, 122, 204, 0.4);
  }

  .vs-brand-text {
    display: flex;
    flex-direction: column;
  }

  .vs-brand-title {
    font-size: 18px;
    font-weight: 700;
    color: var(--vs-text-bright, #ffffff);
  }

  .vs-brand-sub {
    font-size: 12px;
    color: var(--vs-text-dim, #999999);
  }

  .vs-start-close-btn {
    background: transparent;
    border: none;
    color: var(--vs-text-dim, #888888);
    font-size: 18px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .vs-start-close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #ffffff;
  }

  .vs-start-body {
    flex: 1;
    display: grid;
    grid-template-columns: minmax(0, 1.35fr) minmax(360px, 1fr);
    min-height: 0;
    overflow: hidden;
  }

  .vs-start-col-left {
    padding: 24px;
    border-right: 1px solid var(--vs-border, #333333);
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
    overflow-x: hidden;
  }

  .vs-start-col-right {
    padding: 24px;
    background: rgba(0, 0, 0, 0.15);
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
    overflow-y: auto;
    overflow-x: hidden;
  }

  /* Custom VS-style Scrollbars */
  .vs-recent-list::-webkit-scrollbar,
  .vs-start-col-right::-webkit-scrollbar {
    width: 6px;
  }

  .vs-recent-list::-webkit-scrollbar-track,
  .vs-start-col-right::-webkit-scrollbar-track {
    background: transparent;
  }

  .vs-recent-list::-webkit-scrollbar-thumb,
  .vs-start-col-right::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 3px;
  }

  .vs-recent-list::-webkit-scrollbar-thumb:hover,
  .vs-start-col-right::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  .vs-col-title {
    font-size: 15px;
    font-weight: 700;
    color: var(--vs-text-bright, #ffffff);
    margin-bottom: 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .vs-clear-history-btn {
    background: transparent;
    border: none;
    color: var(--vs-text-dim, #888888);
    font-size: 11px;
    cursor: pointer;
    text-decoration: underline;
  }

  .vs-clear-history-btn:hover {
    color: #ef4444;
  }

  .vs-search-box {
    position: relative;
    margin-bottom: 16px;
    display: flex;
    align-items: center;
  }

  .vs-search-icon {
    position: absolute;
    left: 10px;
    font-size: 12px;
    opacity: 0.6;
    pointer-events: none;
  }

  .vs-search-input {
    width: 100%;
    padding: 8px 30px 8px 30px;
    background: var(--vs-bg-2, #252526);
    border: 1px solid var(--vs-border, #3c3c3c);
    border-radius: 4px;
    color: var(--vs-text, #cccccc);
    font-size: 12px;
    box-sizing: border-box;
  }

  .vs-search-input:focus {
    outline: none;
    border-color: var(--vs-accent, #007acc);
  }

  .vs-search-clear {
    position: absolute;
    right: 8px;
    background: transparent;
    border: none;
    color: #888;
    font-size: 12px;
    cursor: pointer;
  }

  .vs-recent-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-right: 6px;
  }

  .vs-recent-empty {
    padding: 32px 16px;
    text-align: center;
    color: var(--vs-text-dim, #777777);
    font-size: 13px;
    line-height: 1.5;
  }

  .vs-recent-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    background: var(--vs-bg-2, #252526);
    border: 1px solid transparent;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
    min-width: 0;
  }

  .vs-recent-card:hover,
  .vs-recent-card.selected {
    background: var(--vs-hover, #2a2d2e);
    border-color: var(--vs-accent, #007acc);
  }

  .vs-recent-icon {
    font-size: 18px;
    opacity: 0.8;
    flex-shrink: 0;
  }

  .vs-recent-icon.pinned {
    opacity: 1;
    filter: drop-shadow(0 0 4px rgba(234, 179, 8, 0.4));
  }

  .vs-recent-info {
    flex: 1;
    min-width: 0;
  }

  .vs-recent-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--vs-text-bright, #ffffff);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .vs-recent-meta {
    font-size: 11px;
    color: var(--vs-text-dim, #888888);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .vs-recent-time {
    font-size: 11px;
    color: var(--vs-text-dim, #777777);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .vs-recent-actions {
    display: flex;
    gap: 4px;
    opacity: 0.3;
    transition: opacity 0.15s ease;
    flex-shrink: 0;
  }

  .vs-recent-card:hover .vs-recent-actions,
  .vs-recent-card.selected .vs-recent-actions {
    opacity: 1;
  }

  .vs-action-icon {
    background: transparent;
    border: none;
    font-size: 12px;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 3px;
    color: #888888;
  }

  .vs-action-icon:hover {
    background: rgba(255, 255, 255, 0.15);
    color: #ffffff;
  }

  .vs-action-icon.remove:hover {
    background: rgba(239, 68, 68, 0.25);
    color: #ef4444;
  }

  .vs-action-cards {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .vs-action-card {
    position: relative;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 14px;
    background: var(--vs-bg-2, #252526);
    border: 1px solid var(--vs-border, #333333);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
    flex-shrink: 0;
    min-width: 0;
  }

  .vs-action-card.primary {
    border-color: rgba(0, 122, 204, 0.45);
    background: linear-gradient(135deg, rgba(0, 122, 204, 0.12), rgba(30, 30, 30, 0.7));
  }

  .vs-action-card.primary .vs-card-icon {
    background: linear-gradient(135deg, #007acc, #005999);
    color: #ffffff;
    box-shadow: 0 2px 10px rgba(0, 122, 204, 0.4);
  }

  .vs-action-card.accent {
    border-color: rgba(16, 185, 129, 0.45);
    background: linear-gradient(135deg, rgba(16, 185, 129, 0.1), rgba(30, 30, 30, 0.7));
  }

  .vs-action-card.accent .vs-card-icon {
    background: linear-gradient(135deg, #10b981, #059669);
    color: #ffffff;
    box-shadow: 0 2px 10px rgba(16, 185, 129, 0.35);
  }

  .vs-action-card:hover:not(:disabled) {
    background: var(--vs-hover, #2d2d30);
    border-color: #38bdf8;
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(0, 122, 204, 0.25);
  }

  .vs-action-card.primary:hover:not(:disabled) {
    background: linear-gradient(135deg, rgba(0, 122, 204, 0.22), rgba(40, 40, 45, 0.8));
    border-color: #38bdf8;
    box-shadow: 0 6px 20px rgba(0, 122, 204, 0.35);
  }

  .vs-action-card:disabled {
    opacity: 0.45;
    cursor: not-allowed;
    pointer-events: none;
    filter: grayscale(0.5);
  }

  .vs-card-icon {
    font-size: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 38px;
    height: 38px;
    background: rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    flex-shrink: 0;
    transition: transform 0.2s ease;
  }

  .vs-action-card:hover:not(:disabled) .vs-card-icon {
    transform: scale(1.05);
  }

  .vs-card-content {
    flex: 1;
    min-width: 0;
  }

  .vs-card-title {
    font-size: 14px;
    font-weight: 700;
    color: var(--vs-text-bright, #ffffff);
    margin-bottom: 3px;
    white-space: normal;
    word-break: break-word;
  }

  .vs-card-sub {
    font-size: 11.5px;
    color: var(--vs-text-dim, #999999);
    line-height: 1.4;
    white-space: normal;
    word-break: break-word;
  }

  .vs-card-arrow {
    font-size: 14px;
    color: var(--vs-text-dim, #666666);
    transition: transform 0.15s ease, color 0.15s ease;
    flex-shrink: 0;
  }

  .vs-action-card:hover:not(:disabled) .vs-card-arrow {
    transform: translateX(4px);
    color: #38bdf8;
  }

  .vs-start-footer {
    padding: 12px 24px;
    background: var(--vs-panel-header-bg, #252526);
    border-top: 1px solid var(--vs-border, #333333);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .vs-show-start-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--vs-text-dim, #999999);
    cursor: pointer;
  }

  .vs-show-start-toggle input {
    cursor: pointer;
  }

  .vs-footer-btn {
    background: transparent;
    border: 1px solid var(--vs-border, #444444);
    color: var(--vs-text, #cccccc);
    padding: 8px 16px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .vs-footer-btn:hover {
    background: var(--vs-hover, #3e3e42);
    color: #ffffff;
    border-color: var(--vs-accent, #007acc);
  }

  /* Submodal styles */
  .vs-submodal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 9500;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .vs-submodal {
    width: 520px;
    background: var(--vs-bg, #1e1e1e);
    border: 1px solid var(--vs-border, #444444);
    border-radius: 8px;
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.7);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .vs-submodal-sm {
    width: 420px;
  }

  .vs-submodal-header {
    padding: 14px 20px;
    background: var(--vs-panel-header-bg, #252526);
    border-bottom: 1px solid var(--vs-border, #333333);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .vs-submodal-header h3 {
    margin: 0;
    font-size: 15px;
    font-weight: 700;
    color: #ffffff;
  }

  .vs-submodal-body {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .vs-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .vs-field label,
  .vs-field-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--vs-text, #cccccc);
  }

  .vs-input {
    padding: 8px 12px;
    background: var(--vs-bg-2, #252526);
    border: 1px solid var(--vs-border, #3c3c3c);
    border-radius: 4px;
    color: #ffffff;
    font-size: 13px;
  }

  .vs-input:focus {
    outline: none;
    border-color: var(--vs-accent, #007acc);
  }

  .vs-template-options {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .vs-template-card {
    display: flex;
    align-items: flex-start;
    gap: 14px;
    padding: 14px;
    background: var(--vs-bg-2, #252526);
    border: 1px solid var(--vs-border, #333333);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.18s ease;
  }

  .vs-template-card:hover {
    border-color: var(--vs-accent, #007acc);
    background: rgba(255, 255, 255, 0.03);
  }

  .vs-template-card.selected {
    border-color: #38bdf8;
    background: linear-gradient(135deg, rgba(0, 122, 204, 0.15), rgba(0, 122, 204, 0.05));
    box-shadow: 0 0 12px rgba(0, 122, 204, 0.25);
  }

  .vs-tmpl-radio {
    font-size: 16px;
    color: #38bdf8;
    margin-top: 1px;
  }

  .vs-tmpl-title {
    font-size: 13px;
    font-weight: 700;
    color: #ffffff;
  }

  .vs-tmpl-desc {
    font-size: 11px;
    color: var(--vs-text-dim, #888888);
    margin-top: 2px;
  }

  .vs-submodal-footer {
    padding: 12px 20px;
    background: var(--vs-panel-header-bg, #252526);
    border-top: 1px solid var(--vs-border, #333333);
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .vs-btn-sec {
    background: transparent;
    border: 1px solid var(--vs-border, #444444);
    color: var(--vs-text, #cccccc);
    padding: 6px 14px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .vs-btn-pri {
    background: var(--vs-accent, #007acc);
    border: none;
    color: #ffffff;
    padding: 6px 16px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .vs-btn-pri:hover {
    background: #005999;
  }

  .vs-btn-danger {
    background: #dc2626;
    border: none;
    color: #ffffff;
    padding: 6px 16px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }

  .vs-btn-danger:hover {
    background: #b91c1c;
  }
</style>
