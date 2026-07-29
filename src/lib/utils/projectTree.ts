import type { FormDef, ProjectNode, ProjectNodeKind, ScadaProject } from "$lib/types";
import { defaultProjectDesignSystem, normalizeProjectDesignSystem } from "$lib/utils/designSystem";
import { ensureMainFormExists } from "$lib/utils/screenProtection";

export const CURRENT_SCHEMA = 3;

const DOC_KINDS: ProjectNodeKind[] = ["script", "note", "markdown", "variables", "style", "component"];

export function uid(prefix = "n"): string {
  return `${prefix}_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 7)}`;
}

export function isDocKind(kind: ProjectNodeKind): boolean {
  return DOC_KINDS.includes(kind);
}

export function iconFor(kind: ProjectNodeKind): string {
  switch (kind) {
    case "folder":
      return "📁";
    case "components_folder":
      return "🧩";
    case "component":
      return "⚙️";
    case "screen":
      return "🗂";
    case "variables":
      return "🏷";
    case "script":
      return "📜";
    case "note":
      return "📝";
    case "markdown":
      return "MD";
    case "image":
      return "🖼️";
    case "style":
      return "🎨";
    default:
      return "•";
  }
}

export function defaultContent(kind: ProjectNodeKind, name: string): string {
  switch (kind) {
    case "script":
      return `# ProScada action script — ${name}
# One statement per line. JavaScript is not executed.
#
# Actions:
#   writeTag "tag.id" <number>
#   ackAlarm "alarm.id"
#   navigate "form.id"
#   log "message"
#
# Guarded statement:
#   if "tag.id" >= 100 then writeTag "other.tag" 0

log "script fired"
`;
    case "style":
      return `/* ProScada Style Sheet — ${name} */
.custom-panel {
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
`;
    case "note":
      return `Note: ${name}\n\n`;
    case "markdown":
      return `# ${name}\n\n`;
    default:
      return "";
  }
}

export function defaultExt(kind: ProjectNodeKind): string {
  switch (kind) {
    case "script":
      return ".js";
    case "style":
      return ".css";
    case "note":
      return ".txt";
    case "markdown":
      return ".md";
    case "screen":
      return ".form";
    case "variables":
      return ".tags";
    default:
      return "";
  }
}

function isChildOfFolder(tree: ProjectNode[], folderId: string, targetRootFolderId: string): boolean {
  const folders = new Map(tree.filter((node) => node.kind === "folder").map((node) => [node.id, node]));
  let current = folders.get(folderId);
  const visited = new Set<string>();

  while (current) {
    if (visited.has(current.id)) return false;
    visited.add(current.id);
    if (current.id === targetRootFolderId) return true;
    if (current.parent_id == null) return false;
    current = folders.get(current.parent_id);
  }

  return false;
}

/** Ensure project has a usable tree; migrate legacy flat forms, deduplicate system folders, and guarantee all root folders and screens. */
export function ensureProjectTree(p: ScadaProject): ScadaProject {
  let tree = [...(p.tree ?? [])].map((node) => ({
    ...node,
    parent_id: node.parent_id ?? null,
  }));
  const guaranteedForms = ensureMainFormExists(Array.isArray(p.forms) ? p.forms : []);
  const normalized = {
    ...p,
    forms: guaranteedForms,
    alarm_groups: Array.isArray(p.alarm_groups) ? p.alarm_groups : [],
    component_templates: Array.isArray(p.component_templates) ? p.component_templates : [],
    design_system: normalizeProjectDesignSystem(p.design_system),
  };

  // 1. Deduplicate system root folders (Screens, Scripts, Styles, Images, Documents)
  const systemNames = ["screens", "scripts", "styles", "images", "documents"];
  const canonicalFolderIds: Record<string, string> = {};
  const duplicateIdMap: Record<string, string> = {};
  const duplicateFolderIds = new Set<string>();

  for (const rawName of systemNames) {
    const canonicalName = rawName.charAt(0).toUpperCase() + rawName.slice(1);
    const matches = tree.filter(
      (n) =>
        n.kind === "folder" &&
        n.parent_id == null &&
        (n.name.toLowerCase() === rawName || (rawName === "documents" && n.name.toLowerCase() === "docs")),
    );

    if (matches.length > 0) {
      const canonical = matches[0];
      canonical.name = canonicalName;
      canonical.parent_id = null;
      if (rawName === "screens") {
        canonical.collapsed = false; // Always force Screens to be expanded
      }
      canonicalFolderIds[rawName] = canonical.id;

      for (let i = 1; i < matches.length; i++) {
        duplicateIdMap[matches[i].id] = canonical.id;
        duplicateFolderIds.add(matches[i].id);
      }
    }
  }

  // Re-parent nodes that were pointing to duplicate root system folders
  tree = tree.map((node) => {
    if (node.parent_id && duplicateIdMap[node.parent_id]) {
      return {
        ...node,
        parent_id: duplicateIdMap[node.parent_id],
      };
    }
    return node;
  });

  // Remove the duplicate root system folders
  tree = tree.filter((n) => !duplicateFolderIds.has(n.id));

  // 2. Guarantee that each system root folder exists
  for (const rawName of systemNames) {
    const canonicalName = rawName.charAt(0).toUpperCase() + rawName.slice(1);
    let canonical = tree.find(
      (n) => n.kind === "folder" && n.parent_id == null && n.name.toLowerCase() === rawName,
    );
    if (!canonical) {
      const newId = uid("fld");
      canonical = {
        id: newId,
        parent_id: null,
        kind: "folder",
        name: canonicalName,
        order: nextOrder(tree, null),
        collapsed: false,
      };
      if (rawName === "screens") {
        tree.unshift(canonical);
      } else {
        tree.push(canonical);
      }
    } else if (rawName === "screens") {
      canonical.collapsed = false;
    }
    canonicalFolderIds[rawName] = canonical.id;
  }

  const screensFolderId = canonicalFolderIds["screens"];

  // 3. Guarantee root 'Variables' item
  const hasVarsItem = tree.some((n) => n.kind === "variables");
  if (!hasVarsItem) {
    tree.push({
      id: uid("var"),
      parent_id: null,
      kind: "variables",
      name: "Variables",
      order: nextOrder(tree, null),
      content: "",
    });
  }

  // 4. Synchronize EVERY form in p.forms with tree — ref_id is canonical key
  for (let i = 0; i < normalized.forms.length; i++) {
    const f = normalized.forms[i];
    let existingNodeIndex = tree.findIndex(
      (n) => n.kind === "screen" && n.ref_id === f.id,
    );
    if (existingNodeIndex < 0) {
      existingNodeIndex = tree.findIndex(
        (n) => n.kind === "screen" && !n.ref_id && n.name === f.name,
      );
    }
    if (existingNodeIndex >= 0) {
      const existing = tree[existingNodeIndex];
      // A screen must be parented either directly to Screens folder or to a subfolder inside Screens
      const parentValid = !!existing.parent_id && isChildOfFolder(tree, existing.parent_id, screensFolderId);
      tree[existingNodeIndex] = {
        ...existing,
        ref_id: f.id,
        name: f.name,
        parent_id: parentValid ? existing.parent_id : screensFolderId,
      };
    } else {
      tree.push({
        id: uid("scr"),
        parent_id: screensFolderId,
        kind: "screen",
        name: f.name,
        order: i,
        ref_id: f.id,
      });
    }
  }

  // 5. Remove screen nodes whose ref_id no longer exists in p.forms
  const formIds = new Set(normalized.forms.map((f) => f.id));
  tree = tree.filter((n) => {
    if (n.kind === "screen" && n.ref_id) {
      return formIds.has(n.ref_id);
    }
    return true;
  });

  // 6. Guarantee root 'Components' folder node
  let componentsFolder = tree.find(
    (n) => n.kind === "components_folder" || (n.kind === "folder" && n.parent_id == null && n.name.toLowerCase() === "komponenty"),
  );
  if (!componentsFolder) {
    componentsFolder = {
      id: uid("cmpfld"),
      parent_id: null,
      kind: "components_folder",
      name: "Komponenty",
      order: nextOrder(tree, null),
      collapsed: false,
    };
    tree.push(componentsFolder);
  } else {
    componentsFolder.kind = "components_folder";
    componentsFolder.name = "Komponenty";
  }

  // 7. Synchronize component_templates with tree
  const componentTemplates = normalized.component_templates ?? [];
  const templateIds = new Set(componentTemplates.map((t) => t.id));

  for (let i = 0; i < componentTemplates.length; i++) {
    const tmpl = componentTemplates[i];
    let existingNodeIndex = tree.findIndex(
      (n) => n.kind === "component" && n.ref_id === tmpl.id,
    );
    if (existingNodeIndex >= 0) {
      tree[existingNodeIndex] = {
        ...tree[existingNodeIndex],
        name: tmpl.name,
        parent_id: componentsFolder.id,
      };
    } else {
      tree.push({
        id: uid("cmpnode"),
        parent_id: componentsFolder.id,
        kind: "component",
        name: tmpl.name,
        order: i,
        ref_id: tmpl.id,
      });
    }
  }

  // 8. Remove component nodes whose ref_id no longer exists in component_templates
  tree = tree.filter((n) => {
    if (n.kind === "component" && n.ref_id) {
      return templateIds.has(n.ref_id);
    }
    return true;
  });

  tree = tree.map((n) => ({
    ...n,
    parent_id: n.parent_id ?? null,
  }));

  return {
    ...normalized,
    schema_version: Math.max(p.schema_version || 1, CURRENT_SCHEMA),
    tree,
  };
}

export function createEmptyProject(name = "New Project"): ScadaProject {
  const formId = uid("form");
  const form: FormDef = {
    id: formId,
    name: "Main",
    width: 1040,
    height: 700,
    background: "#F4F5F7",
    grid: 8,
    widgets: [],
  };
  const base: ScadaProject = {
    schema_version: CURRENT_SCHEMA,
    id: uid("proj"),
    name,
    description: "",
    devices: [],
    tags: [],
    forms: [form],
    alarms: [],
    alarm_groups: [],
    design_system: defaultProjectDesignSystem(),
    component_templates: [],
    tree: [],
    content_hash: "",
  };
  return ensureProjectTree(base);
}

export function childrenOf(tree: ProjectNode[], parentId: string | null): ProjectNode[] {
  const targetParent = parentId ?? null;
  return tree
    .filter((n) => (n.parent_id ?? null) === targetParent)
    .sort((a, b) => a.order - b.order || a.name.localeCompare(b.name));
}

export function findNode(tree: ProjectNode[], id: string): ProjectNode | undefined {
  return tree.find((n) => n.id === id);
}

export function collectDescendantIds(tree: ProjectNode[], rootId: string): string[] {
  const out: string[] = [rootId];
  const walk = (pid: string) => {
    for (const c of tree.filter((n) => (n.parent_id ?? null) === pid)) {
      out.push(c.id);
      if (c.kind === "folder") walk(c.id);
    }
  };
  walk(rootId);
  return out;
}

export function nextOrder(tree: ProjectNode[], parentId: string | null): number {
  const sibs = childrenOf(tree, parentId);
  return sibs.length === 0 ? 0 : Math.max(...sibs.map((s) => s.order)) + 1;
}

export function isAncestor(
  tree: ProjectNode[],
  maybeAncestorId: string,
  nodeId: string,
): boolean {
  let cur = findNode(tree, nodeId);
  while (cur?.parent_id) {
    if (cur.parent_id === maybeAncestorId) return true;
    cur = findNode(tree, cur.parent_id);
  }
  return false;
}

/**
 * Validate only the minimum envelope needed before handing an imported project
 * to the engine. This function must not normalize, migrate or otherwise mutate
 * the payload: `content_hash` authenticates the incoming representation and the
 * engine must see it before any UI transformation.
 */
export function validateImportedProjectEnvelope(raw: unknown): ScadaProject {
  if (!raw || typeof raw !== "object") throw new Error("Invalid project file");
  const o = raw as Record<string, unknown>;
  if (typeof o.id !== "string" || typeof o.name !== "string") {
    throw new Error("Project must have id and name");
  }
  if (!Array.isArray(o.forms)) throw new Error("Project must have forms[]");
  return o as unknown as ScadaProject;
}

/**
 * Normalize a project only after the engine has accepted and verified it.
 * Preserve `content_hash`; clearing an incoming hash would turn integrity
 * verification into an opt-out controlled by the file being imported.
 */
export function normalizeImportedProject(raw: unknown): ScadaProject {
  const source = validateImportedProjectEnvelope(raw);
  const p: ScadaProject = {
    ...source,
    devices: Array.isArray(source.devices) ? source.devices : [],
    tags: Array.isArray(source.tags) ? source.tags : [],
    forms: Array.isArray(source.forms) ? source.forms : [],
    alarms: Array.isArray(source.alarms) ? source.alarms : [],
    alarm_groups: Array.isArray(source.alarm_groups) ? source.alarm_groups : [],
    component_templates: Array.isArray(source.component_templates)
      ? source.component_templates
      : [],
    design_system: normalizeProjectDesignSystem(source.design_system),
    tree: Array.isArray(source.tree) ? source.tree : [],
    description: source.description ?? "",
    content_hash: typeof source.content_hash === "string" ? source.content_hash : "",
    schema_version: Number(source.schema_version) || CURRENT_SCHEMA,
  };
  return ensureProjectTree(p);
}

export interface ProjectImageItem {
  id: string;
  name: string;
  path: string;
  content: string;
}

export function getNodePath(tree: ProjectNode[], nodeId: string): string {
  const parts: string[] = [];
  let cur = findNode(tree, nodeId);
  while (cur) {
    parts.unshift(cur.name);
    if (!cur.parent_id) break;
    cur = findNode(tree, cur.parent_id);
  }
  return parts.join("/");
}

export function collectProjectImages(tree: ProjectNode[]): ProjectImageItem[] {
  return tree
    .filter((n) => n.kind === "image" && !!n.content)
    .map((n) => ({
      id: n.id,
      name: n.name,
      path: getNodePath(tree, n.id),
      content: n.content ?? "",
    }))
    .sort((a, b) => a.path.localeCompare(b.path));
}
