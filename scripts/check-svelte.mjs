import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const cli = join(root, "node_modules", "svelte-check", "bin", "svelte-check");
const result = spawnSync(
  process.execPath,
  [cli, "--tsconfig", "./tsconfig.json", "--output", "machine", "--no-color"],
  {
    cwd: root,
    encoding: "utf8",
  },
);

const output = `${result.stdout ?? ""}${result.stderr ?? ""}`;
process.stdout.write(output);

if (result.error) {
  console.error(`Cannot start svelte-check: ${result.error.message}`);
  process.exit(1);
}
if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

const warningLines = output.split(/\r?\n/).filter((line) => /\sWARNING\s/.test(line));

if (warningLines.length > 0) {
  console.error(`Svelte check failed: expected 0 warnings, found ${warningLines.length}.`);
  process.exit(1);
}

console.log("Svelte check OK: 0 errors and 0 warnings.");
