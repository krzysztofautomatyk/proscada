import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseDocument } from "yaml";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const errors = [];
const expectedSkills = [
  "proscada-widget",
  "proscada-modbus",
  "proscada-alarm",
  "proscada-component-template",
  "proscada-designer",
  "proscada-docs",
  "proscada-validation",
  "proscada-security",
];
const expectedInstructions = [
  "frontend.instructions.md",
  "rust.instructions.md",
  "docs.instructions.md",
  "automation.instructions.md",
  "tauri-security.instructions.md",
];
const expectedAgents = [
  "scada-architect.agent.md",
  "widget-engineer.agent.md",
  "ot-safety-reviewer.agent.md",
  "documentation-maintainer.agent.md",
  "ci-validator.agent.md",
  "expert-panel.agent.md",
];
const requiredFiles = [
  "AGENTS.md",
  "CONTRIBUTING.md",
  "llms.txt",
  "public/llms.txt",
  ".node-version",
  "rust-toolchain.toml",
  ".github/copilot-instructions.md",
  ".github/workflows/copilot-setup-steps.yml",
  ".github/workflows/ci.yml",
  ".github/skills/README.md",
  ".github/agents/README.md",
  ".github/pull_request_template.md",
  ".github/SECURITY.md",
  ".github/CODEOWNERS",
  ".github/dependabot.yml",
  "docs/ai/README.md",
  "docs/ai/context-map.md",
  "docs/ai/expert-panel.md",
  "docs/ai/workflow.md",
  "docs/ai/guardrails.md",
  "docs/ai/cloud-agent.md",
  "docs/ai/skills-catalog.md",
];

function text(relativePath) {
  const path = join(root, relativePath);
  if (!existsSync(path)) {
    errors.push(`Missing AI configuration: ${relativePath}`);
    return "";
  }
  return readFileSync(path, "utf8");
}

function frontmatter(content, relativePath) {
  const match = content.match(/^---\r?\n([\s\S]*?)\r?\n---/);
  if (!match) {
    errors.push(`${relativePath}: missing YAML frontmatter`);
    return {};
  }
  const document = parseDocument(match[1], {
    prettyErrors: true,
    uniqueKeys: true,
  });
  for (const error of document.errors) {
    errors.push(`${relativePath}: invalid YAML frontmatter: ${error.message}`);
  }
  const data = document.toJS();
  if (!data || typeof data !== "object" || Array.isArray(data)) {
    errors.push(`${relativePath}: frontmatter must be a mapping`);
    return {};
  }
  return data;
}

for (const path of requiredFiles) text(path);

const instructionsRoot = join(root, ".github", "instructions");
for (const name of expectedInstructions) {
  const relativePath = `.github/instructions/${name}`;
  const content = text(relativePath);
  const fm = frontmatter(content, relativePath);
  if (typeof fm.applyTo !== "string" || !fm.applyTo.trim()) {
    errors.push(`${relativePath}: missing applyTo`);
  }
}

const agentsRoot = join(root, ".github", "agents");
for (const name of expectedAgents) {
  const relativePath = `.github/agents/${name}`;
  const content = text(relativePath);
  const fm = frontmatter(content, relativePath);
  if (typeof fm.name !== "string" || !/^[a-z0-9-]+$/.test(fm.name)) {
    errors.push(`${relativePath}: invalid name`);
  }
  if (typeof fm.description !== "string" || !fm.description.trim()) {
    errors.push(`${relativePath}: missing description`);
  }
  if (!Array.isArray(fm.tools) || fm.tools.length === 0) {
    errors.push(`${relativePath}: missing explicit tools`);
  }
  if (fm.name !== name.replace(".agent.md", "")) {
    errors.push(`${relativePath}: agent name must match filename`);
  }
  if (fm["user-invocable"] !== true) {
    errors.push(`${relativePath}: user-invocable must be true`);
  }
  const allowedTools = new Set(["read", "search", "edit", "execute"]);
  if (Array.isArray(fm.tools) && fm.tools.some((tool) => !allowedTools.has(tool))) {
    errors.push(`${relativePath}: contains an unsupported tool`);
  }
}

for (const reviewer of ["ot-safety-reviewer.agent.md"]) {
  const fm = frontmatter(
    text(`.github/agents/${reviewer}`),
    `.github/agents/${reviewer}`,
  );
  if (Array.isArray(fm.tools) && fm.tools.some((tool) => tool === "edit" || tool === "execute")) {
    errors.push(`.github/agents/${reviewer}: read-only reviewer has write/execute tools`);
  }
  if (fm["disable-model-invocation"] !== true) {
    errors.push(`.github/agents/${reviewer}: reviewer must disable model invocation`);
  }
}
const ciFm = frontmatter(
  text(".github/agents/ci-validator.agent.md"),
  ".github/agents/ci-validator.agent.md",
);
if (Array.isArray(ciFm.tools) && ciFm.tools.includes("edit")) {
  errors.push(".github/agents/ci-validator.agent.md: validator must not edit");
}
if (ciFm["disable-model-invocation"] !== true) {
  errors.push(".github/agents/ci-validator.agent.md: validator must disable model invocation");
}

const skillsRoot = join(root, ".github", "skills");
const actualSkillDirs = existsSync(skillsRoot)
  ? readdirSync(skillsRoot, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name)
      .sort()
  : [];
if (actualSkillDirs.length !== expectedSkills.length) {
  errors.push(`Expected ${expectedSkills.length} skill folders, found ${actualSkillDirs.length}`);
}
for (const name of expectedSkills) {
  const relativePath = `.github/skills/${name}/SKILL.md`;
  const content = text(relativePath);
  const fm = frontmatter(content, relativePath);
  if (fm.name !== name) {
    errors.push(`${relativePath}: frontmatter name must match folder`);
  }
  if (typeof fm.description !== "string" || !fm.description.trim()) {
    errors.push(`${relativePath}: missing description`);
  }
  if (!/validate|cargo|npm run/i.test(content)) {
    errors.push(`${relativePath}: missing validation commands`);
  }
}

const markdownRoots = [
  join(root, "AGENTS.md"),
  join(root, "CONTRIBUTING.md"),
  join(root, ".github", "copilot-instructions.md"),
  join(root, ".github", "pull_request_template.md"),
  join(root, ".github", "SECURITY.md"),
  join(root, ".github", "skills", "README.md"),
  join(root, ".github", "agents", "README.md"),
  ...expectedInstructions.map((name) => join(instructionsRoot, name)),
  ...expectedAgents.map((name) => join(agentsRoot, name)),
  ...expectedSkills.map((name) => join(skillsRoot, name, "SKILL.md")),
];
for (const path of markdownRoots.filter(existsSync)) {
  const content = readFileSync(path, "utf8");
  const lines = content.split(/\r?\n/).length;
  if (statSync(path).size > 16 * 1024) errors.push(`${relative(root, path)}: exceeds 16 KB`);
  if (lines > 220) errors.push(`${relative(root, path)}: exceeds 220 lines`);
  if (/[A-Za-z]:\\Users\\/.test(content)) {
    errors.push(`${relative(root, path)}: contains a machine-specific absolute path`);
  }
  if (/src-tauri\\Cargo\.toml/.test(content)) {
    errors.push(`${relative(root, path)}: uses a Windows-only Cargo manifest path`);
  }
  if (/AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{20,}|BEGIN (RSA |EC )?PRIVATE KEY/.test(content)) {
    errors.push(`${relative(root, path)}: contains a secret-like value`);
  }
  for (const match of content.matchAll(/\[[^\]]+\]\(([^)]+)\)/g)) {
    const target = match[1].split("#")[0].trim();
    if (!target || /^(https?:|mailto:)/i.test(target)) continue;
    const resolved = resolve(dirname(path), decodeURIComponent(target));
    if (!existsSync(resolved)) {
      errors.push(`${relative(root, path)}: broken link ${match[1]}`);
    }
  }
}

function validateWorkflow(relativePath) {
  const content = text(relativePath);
  if (/pull_request_target\s*:/.test(content)) {
    errors.push(`${relativePath}: pull_request_target is forbidden`);
  }
  if (!/permissions:\s*\r?\n\s+contents:\s*read/.test(content)) {
    errors.push(`${relativePath}: missing minimal contents: read permission`);
  }
  for (const match of content.matchAll(/uses:\s*([^@\s]+)@([^\s#]+)/g)) {
    if (match[1].startsWith("./")) continue;
    if (!/^[a-f0-9]{40}$/.test(match[2])) {
      errors.push(`${relativePath}: action ${match[1]} is not pinned to a full SHA`);
    }
  }
  return content;
}

const setup = validateWorkflow(".github/workflows/copilot-setup-steps.yml");
const jobsBlock = setup.split(/\r?\njobs:\s*\r?\n/)[1] ?? "";
const setupJobs = [...jobsBlock.matchAll(/^  ([A-Za-z0-9_-]+):\s*$/gm)].map(
  (match) => match[1],
);
if (setupJobs.length !== 1 || setupJobs[0] !== "copilot-setup-steps") {
  errors.push("copilot-setup-steps.yml must contain exactly one copilot-setup-steps job");
}
for (const token of [
  "npm ci",
  "cargo fetch --locked",
  "validate:ai",
  "validate:yaml",
  "test:pump-template",
  "timeout-minutes: 45",
]) {
  if (!setup.includes(token)) errors.push(`copilot-setup-steps.yml: missing ${token}`);
}

const ci = validateWorkflow(".github/workflows/ci.yml");
for (const token of [
  "npm run check",
  "npm run validate:widgets",
  "npm run validate:docs",
  "npm run validate:ai",
  "npm run validate:yaml",
  // The whole suite must run, not a single hand-picked test file: a red test
  // outside the gate is a test nobody sees.
  "npm test",
  "npm run build",
  "cargo fmt",
  "cargo clippy",
  "cargo test",
  "cargo build",
]) {
  if (!ci.includes(token)) errors.push(`ci.yml: missing ${token}`);
}

const agents = text("AGENTS.md");
if (!agents.includes(".github/skills/")) errors.push("AGENTS.md: missing skills discovery");
if (!agents.includes("nie mogą łączyć się z PLC")) errors.push("AGENTS.md: missing OT isolation");
if (!text("package.json").includes('"validate:ai"')) errors.push("package.json: missing validate:ai");
if (!text("package.json").includes('"validate:yaml"')) errors.push("package.json: missing validate:yaml");
if (!text("package.json").includes('"test:pump-template"')) {
  errors.push("package.json: missing test:pump-template");
}
if (text(".node-version").trim() !== "22") errors.push(".node-version must pin Node 22");
if (!text("rust-toolchain.toml").includes('channel = "1.88.0"')) {
  errors.push("rust-toolchain.toml must pin Rust 1.88.0");
}
const publicLlms = text("public/llms.txt");
if (!publicLlms.startsWith("# ProScada")) errors.push("public/llms.txt: missing H1");
if (!/^> .+/m.test(publicLlms)) errors.push("public/llms.txt: missing summary blockquote");
if (!text("llms.txt").includes("[Toolbox 45/45]")) {
  errors.push("llms.txt: Toolbox total must be explicit as 45/45");
}
if (!text("docs/toolbox/README.md").startsWith("# Toolbox — 45/45 pozycji")) {
  errors.push("docs/toolbox/README.md: missing explicit 45/45 total");
}
if (!text(".github/CODEOWNERS").includes("@krzysztofautomatyk")) {
  errors.push(".github/CODEOWNERS: missing repository owner");
}
if (!text(".github/dependabot.yml").includes("package-ecosystem: cargo")) {
  errors.push(".github/dependabot.yml: missing Cargo updates");
}

if (errors.length) {
  console.error(errors.join("\n"));
  process.exit(1);
}

console.log(
  `AI config OK: ${expectedInstructions.length} path instructions, ${expectedAgents.length} expert agents, ${expectedSkills.length} skills and 2 pinned workflows.`,
);
