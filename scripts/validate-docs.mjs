import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, extname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const docsRoot = join(root, "docs");
const toolboxRoot = join(docsRoot, "toolbox");
const controlsRoot = join(toolboxRoot, "controls");
const templatesRoot = join(toolboxRoot, "templates");
const maxBytes = 10 * 1024;
const maxLines = 160;
const staleNames = new Set([
  "ARCHITECTURE.md",
  "COMPLIANCE.md",
  "WATER_TANK_INTEGRATION.md",
  "WIDGET_DYNAMICS.md",
  "00-expert-council.md",
]);
const categoryFolders = {
  Prymitywy: "primitives",
  Zasoby: "assets",
  Wskaźniki: "indicators",
  "Wizualizacja procesu": "process",
  Sterowanie: "commands",
  Wejścia: "inputs",
  "Dane i historia": "data",
  Układ: "layout",
  Nawigacja: "navigation",
  "Informacja i interakcja": "feedback",
  Alarmy: "alarms",
  Narzędzia: "utilities",
};
const genericDocs = [
  "registry-factory.md",
  "common-properties.md",
  "binding-quality.md",
  "styles-fonts.md",
  "dynamics-events.md",
  "definition-of-done.md",
];

function walk(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? walk(path) : [path];
  });
}

function validateItemContent(content, path) {
  const rel = relative(root, path);
  if (!/\| Plik \| `[^`]+` \|/.test(content)) {
    errors.push(`${rel}: missing renderer file metadata`);
  }
  if (!/## (Najważniejsze pola|Konfiguracja|Format danych)/.test(content)) {
    errors.push(`${rel}: missing configuration section`);
  }
  if (
    !/## (Dane|Zachowanie|Funkcje|Interakcja|Runtime|Stany|Roll-up|Walidacja|Sterowanie|Quality|ACK|Lifecycle|Tryby|Warianty|Zapis|Dynamika|Binding)/.test(
      content,
    )
  ) {
    errors.push(`${rel}: missing behavior section`);
  }
  if (!/## (Zasady|Bezpieczeństwo|Ograniczenia|Status|Dostępność|Quality)/.test(content)) {
    errors.push(`${rel}: missing limitations or safety section`);
  }
}

const markdownFiles = walk(docsRoot).filter((path) => extname(path).toLowerCase() === ".md");
const filesToValidate = [join(root, "README.md"), ...markdownFiles];
const errors = [];
if (markdownFiles.length < 20) errors.push(`Expected modular documentation, found only ${markdownFiles.length} files`);

const catalogPath = join(
  root,
  "src",
  "lib",
  "components",
  "widgets",
  "registry",
  "catalog.ts",
);
const catalog = readFileSync(catalogPath, "utf8");
const canonicalBlock = catalog.match(
  /const canonicalWidgets: WidgetCatalogItem\[\] = \[([\s\S]*?)\r?\n\];\r?\n\r?\nconst processTemplates/,
);
if (!canonicalBlock) {
  errors.push("Cannot read canonical widget catalog");
}
const controls = canonicalBlock
  ? [...canonicalBlock[1].matchAll(
      /canonicalId:\s*"([^"]+)"[\s\S]*?type:\s*"([^"]+)"[\s\S]*?category:\s*"([^"]+)"/g,
    )].map((match) => ({ id: match[1], type: match[2], category: match[3] }))
  : [];
if (controls.length !== 35) errors.push(`Expected 35 canonical controls, found ${controls.length}`);
const templateBlock = catalog.match(
  /const processTemplates: WidgetCatalogItem\[\] = \[([\s\S]*?)\r?\n\];\r?\n\r?\nexport const WIDGET_CATALOG/,
);
if (!templateBlock) {
  errors.push("Cannot read process template catalog");
}
const templates = templateBlock
  ? [...templateBlock[1].matchAll(
      /canonicalId:\s*"([^"]+)"[\s\S]*?type:\s*"([^"]+)"[\s\S]*?category:\s*"([^"]+)"/g,
    )].map((match) => ({ id: match[1], type: match[2], category: match[3] }))
  : [];
if (templates.length !== 10) errors.push(`Expected 10 Toolbox templates, found ${templates.length}`);

const toolboxIndexPath = join(toolboxRoot, "README.md");
const toolboxIndex = readFileSync(toolboxIndexPath, "utf8");
for (const control of controls) {
  const folder = categoryFolders[control.category];
  if (!folder) {
    errors.push(`No documentation folder mapped for category ${control.category}`);
    continue;
  }
  const docPath = join(controlsRoot, folder, `${control.type}.md`);
  const docLink = `controls/${folder}/${control.type}.md`;
  if (!existsSync(docPath)) {
    errors.push(`${control.id}/${control.type}: missing documentation ${relative(root, docPath)}`);
    continue;
  }
  const content = readFileSync(docPath, "utf8");
  validateItemContent(content, docPath);
  if (!content.includes(`| ID | \`${control.id}\` |`)) {
    errors.push(`${relative(root, docPath)}: missing canonical ID ${control.id}`);
  }
  if (!content.includes(`| Typ | \`${control.type}\` |`)) {
    errors.push(`${relative(root, docPath)}: missing widget type ${control.type}`);
  }
  if (!toolboxIndex.includes(`](${docLink})`)) {
    errors.push(`docs/toolbox/README.md: missing link to ${docLink}`);
  }
}

const controlDocs = walk(controlsRoot).filter((path) => extname(path).toLowerCase() === ".md");
if (controlDocs.length !== 35) {
  errors.push(`Expected exactly 35 per-control documents, found ${controlDocs.length}`);
}
for (const template of templates) {
  const docPath = join(templatesRoot, `${template.type}.md`);
  const docLink = `templates/${template.type}.md`;
  if (!existsSync(docPath)) {
    errors.push(`${template.id}/${template.type}: missing documentation ${relative(root, docPath)}`);
    continue;
  }
  const content = readFileSync(docPath, "utf8");
  validateItemContent(content, docPath);
  if (!content.includes(`| ID | \`${template.id}\` |`)) {
    errors.push(`${relative(root, docPath)}: missing template ID ${template.id}`);
  }
  if (!content.includes(`| Typ | \`${template.type}\` |`)) {
    errors.push(`${relative(root, docPath)}: missing template type ${template.type}`);
  }
  if (!toolboxIndex.includes(`](${docLink})`)) {
    errors.push(`docs/toolbox/README.md: missing link to ${docLink}`);
  }
}
const templateDocs = walk(templatesRoot).filter((path) => extname(path).toLowerCase() === ".md");
if (templateDocs.length !== 10) {
  errors.push(`Expected exactly 10 per-template documents, found ${templateDocs.length}`);
}
for (const name of genericDocs) {
  const path = join(toolboxRoot, "generic", name);
  if (!existsSync(path)) errors.push(`Missing generic Toolbox documentation: ${relative(root, path)}`);
  if (!toolboxIndex.includes(`](generic/${name})`)) {
    errors.push(`docs/toolbox/README.md: missing generic link generic/${name}`);
  }
}

for (const path of filesToValidate) {
  const rel = relative(root, path);
  const bytes = statSync(path).size;
  const content = readFileSync(path, "utf8");
  const lines = content.split(/\r?\n/).length;
  if (!content.trim()) errors.push(`${rel}: empty file`);
  if (bytes > maxBytes) errors.push(`${rel}: ${bytes} bytes exceeds ${maxBytes}`);
  if (lines > maxLines) errors.push(`${rel}: ${lines} lines exceeds ${maxLines}`);
  for (const stale of staleNames) {
    if (content.includes(stale)) errors.push(`${rel}: stale documentation reference ${stale}`);
  }
  for (const match of content.matchAll(/\[[^\]]+\]\(([^)]+)\)/g)) {
    const target = match[1].split("#")[0].trim();
    if (!target || /^(https?:|mailto:)/i.test(target)) continue;
    const resolved = resolve(dirname(path), decodeURIComponent(target));
    if (!existsSync(resolved)) errors.push(`${rel}: broken link ${match[1]}`);
  }
}

if (errors.length) {
  console.error(errors.join("\n"));
  process.exit(1);
}

const largest = markdownFiles
  .map((path) => ({ path: relative(root, path), bytes: statSync(path).size }))
  .sort((a, b) => b.bytes - a.bytes)[0];
console.log(
  `Docs OK: ${markdownFiles.length} modular files, 45/45 Toolbox item docs and ${genericDocs.length} generic Toolbox docs; largest ${largest.path} (${largest.bytes} bytes).`,
);
