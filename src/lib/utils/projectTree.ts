import type { FormDef, ProjectNode, ProjectNodeKind, ScadaProject } from "$lib/types";
import { defaultProjectDesignSystem, normalizeProjectDesignSystem } from "$lib/utils/designSystem";

export const CURRENT_SCHEMA = 3;

const DOC_KINDS: ProjectNodeKind[] = ["script", "note", "markdown", "variables"];

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
    default:
      return "•";
  }
}

export function defaultContent(kind: ProjectNodeKind, name: string): string {
  switch (kind) {
    case "script":
      return `// ProScada HMI script — ${name}
// API: writeTag(id, value), getTag(id), getTagValue(id), log(msg), navigate(formId), ackAlarm(id)
// Event: event = { type, widgetId, formId, tagId }

async function onEvent(event) {
  log("script fired: " + event.type + " @ " + (event.widgetId ?? "?"));
  // Example: await writeTag("wt.sim_en", 1);
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

/** Ensure project has a usable tree; migrate legacy flat forms and guarantee Images root folder. */
export function ensureProjectTree(p: ScadaProject): ScadaProject {
  let tree = [...(p.tree ?? [])];
  const normalized = {
    ...p,
    alarm_groups: Array.isArray(p.alarm_groups) ? p.alarm_groups : [],
    component_templates: Array.isArray(p.component_templates) ? p.component_templates : [],
    design_system: normalizeProjectDesignSystem(p.design_system),
  };

  if (tree.length === 0) {
    const screensId = uid("fld");
    const scriptsId = uid("fld");
    const docsId = uid("fld");
    const imagesId = uid("fld");
    const varsId = uid("var");

    tree = [
      { id: screensId, parent_id: null, kind: "folder", name: "Screens", order: 0 },
      { id: scriptsId, parent_id: null, kind: "folder", name: "Scripts", order: 1 },
      { id: imagesId, parent_id: null, kind: "folder", name: "Images", order: 2 },
      { id: docsId, parent_id: null, kind: "folder", name: "Documents", order: 3 },
      {
        id: varsId,
        parent_id: null,
        kind: "variables",
        name: "Variables",
        order: 4,
        content: "",
      },
    ];

    normalized.forms.forEach((f, i) => {
      tree.push({
        id: uid("scr"),
        parent_id: screensId,
        kind: "screen",
        name: f.name,
        order: i,
        ref_id: f.id,
      });
    });
  }

  // Guarantee that a root 'Images' folder always exists
  const hasImagesFolder = tree.some(
    (n) => n.kind === "folder" && n.parent_id === null && n.name.toLowerCase() === "images",
  );
  if (!hasImagesFolder) {
    tree.push({
      id: uid("fld"),
      parent_id: null,
      kind: "folder",
      name: "Images",
      order: nextOrder(tree, null),
      collapsed: false,
    });
  }

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
  return tree
    .filter((n) => n.parent_id === parentId)
    .sort((a, b) => a.order - b.order || a.name.localeCompare(b.name));
}

export function findNode(tree: ProjectNode[], id: string): ProjectNode | undefined {
  return tree.find((n) => n.id === id);
}

export function collectDescendantIds(tree: ProjectNode[], rootId: string): string[] {
  const out: string[] = [rootId];
  const walk = (pid: string) => {
    for (const c of tree.filter((n) => n.parent_id === pid)) {
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

/** Validate imported JSON shape loosely. */
export function normalizeImportedProject(raw: unknown): ScadaProject {
  if (!raw || typeof raw !== "object") throw new Error("Invalid project file");
  const o = raw as Record<string, unknown>;
  if (typeof o.id !== "string" || typeof o.name !== "string") {
    throw new Error("Project must have id and name");
  }
  if (!Array.isArray(o.forms)) throw new Error("Project must have forms[]");
  const p = o as unknown as ScadaProject;
  p.devices = Array.isArray(p.devices) ? p.devices : [];
  p.tags = Array.isArray(p.tags) ? p.tags : [];
  p.alarms = Array.isArray(p.alarms) ? p.alarms : [];
  p.alarm_groups = Array.isArray(p.alarm_groups) ? p.alarm_groups : [];
  p.component_templates = Array.isArray(p.component_templates) ? p.component_templates : [];
  p.design_system = normalizeProjectDesignSystem(p.design_system);
  p.tree = Array.isArray(p.tree) ? p.tree : [];
  p.description = p.description ?? "";
  p.content_hash = p.content_hash ?? "";
  p.schema_version = Number(p.schema_version) || CURRENT_SCHEMA;
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
