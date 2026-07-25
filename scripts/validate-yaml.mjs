import { readFileSync, readdirSync } from "node:fs";
import { extname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseDocument } from "yaml";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const githubRoot = join(root, ".github");

function walk(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? walk(path) : [path];
  });
}

const files = walk(githubRoot).filter((path) =>
  [".yml", ".yaml"].includes(extname(path).toLowerCase()),
);
const errors = [];

for (const path of files) {
  const document = parseDocument(readFileSync(path, "utf8"), {
    prettyErrors: true,
    uniqueKeys: true,
  });
  for (const error of document.errors) {
    errors.push(`${relative(root, path)}: ${error.message}`);
  }
}

if (errors.length) {
  console.error(errors.join("\n"));
  process.exit(1);
}

console.log(`YAML OK: ${files.length} GitHub configuration files parsed successfully.`);

