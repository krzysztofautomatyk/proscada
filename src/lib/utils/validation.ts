import type { ScadaProject } from "$lib/types";
import { WIDGET_CATALOG } from "$lib/components/widgets/registry";
import { parseScript } from "$lib/services/scriptRuntime";

export interface ValidationIssue {
  severity: "error" | "warning";
  path: string;
  message: string;
  targetFormId?: string;
  targetWidgetId?: string;
  targetNodeId?: string;
  targetTagId?: string;
  targetDeviceId?: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationIssue[];
  warnings: ValidationIssue[];
}

export function validateProject(project: ScadaProject | null | undefined): ValidationResult {
  const errors: ValidationIssue[] = [];
  const warnings: ValidationIssue[] = [];

  if (!project) {
    return {
      valid: false,
      errors: [{ severity: "error", path: "project", message: "Project is null or undefined" }],
      warnings: [],
    };
  }

  if (!project.id || typeof project.id !== "string" || !project.id.trim()) {
    errors.push({ severity: "error", path: "project.id", message: "Project must have a non-empty ID" });
  }

  if (!project.name || typeof project.name !== "string" || !project.name.trim()) {
    errors.push({ severity: "error", path: "project.name", message: "Project must have a non-empty Name" });
  }

  // 1. Devices validation
  const deviceIds = new Set<string>();
  (project.devices ?? []).forEach((dev, idx) => {
    const path = `devices/${dev.name || dev.id || idx}`;
    if (!dev.id || deviceIds.has(dev.id)) {
      errors.push({
        severity: "error",
        path: `${path}.id`,
        message: `Duplicate or empty device ID: ${dev.id}`,
        targetDeviceId: dev.id,
      });
    } else {
      deviceIds.add(dev.id);
    }

    if (!dev.name || !dev.name.trim()) {
      warnings.push({
        severity: "warning",
        path: `${path}.name`,
        message: `Device ${dev.id} has no label name`,
        targetDeviceId: dev.id,
      });
    }

    if (!dev.host || typeof dev.host !== "string" || !dev.host.trim()) {
      errors.push({
        severity: "error",
        path: `${path}.host`,
        message: `Device ${dev.id} host address is missing`,
        targetDeviceId: dev.id,
      });
    }

    if (!Number.isInteger(dev.port) || dev.port <= 0 || dev.port > 65535) {
      errors.push({
        severity: "error",
        path: `${path}.port`,
        message: `Device ${dev.id} port must be 1..65535`,
        targetDeviceId: dev.id,
      });
    }

    if (!Number.isInteger(dev.unit_id) || dev.unit_id < 0 || dev.unit_id > 247) {
      errors.push({
        severity: "error",
        path: `${path}.unit_id`,
        message: `Device ${dev.id} Modbus Unit ID must be 0..247`,
        targetDeviceId: dev.id,
      });
    }

    if (dev.poll_ms < 10) {
      warnings.push({
        severity: "warning",
        path: `${path}.poll_ms`,
        message: `Polling interval ${dev.poll_ms}ms is very fast`,
        targetDeviceId: dev.id,
      });
    }
  });

  // 2. Tags validation
  const tagIds = new Set<string>();
  const physicalAddresses = new Set<string>();

  (project.tags ?? []).forEach((tag, idx) => {
    const path = `tags/${tag.name || tag.id || idx}`;
    if (!tag.id || tagIds.has(tag.id)) {
      errors.push({
        severity: "error",
        path: `${path}.id`,
        message: `Duplicate or empty tag ID: ${tag.id}`,
        targetTagId: tag.id,
      });
    } else {
      tagIds.add(tag.id);
    }

    if (tag.scale === 0) {
      errors.push({
        severity: "error",
        path: `${path}.scale`,
        message: `Tag ${tag.id} scale cannot be 0`,
        targetTagId: tag.id,
      });
    }

    if (tag.binding) {
      if (tag.binding.address < 0 || tag.binding.address > 65535) {
        errors.push({
          severity: "error",
          path: `${path}.binding.address`,
          message: `Tag ${tag.id} address must be 0..65535`,
          targetTagId: tag.id,
        });
      }

      if (tag.binding.bit != null) {
        if (!Number.isInteger(tag.binding.bit) || tag.binding.bit < 0 || tag.binding.bit > 15) {
          errors.push({
            severity: "error",
            path: `${path}.binding.bit`,
            message: `Tag ${tag.id} bit index must be 0..15`,
            targetTagId: tag.id,
          });
        }
      }

      // Check physical collision for word registers
      const key = `${tag.device_id}:${tag.binding.table}:${tag.binding.address}`;
      if (tag.binding.bit == null && physicalAddresses.has(key)) {
        warnings.push({
          severity: "warning",
          path: `${path}.binding`,
          message: `Physical address overlap at ${key}`,
          targetTagId: tag.id,
        });
      } else if (tag.binding.bit == null) {
        physicalAddresses.add(key);
      }
    }
  });

  // 3. Forms & Widgets validation
  if (!Array.isArray(project.forms) || project.forms.length === 0) {
    errors.push({ severity: "error", path: "forms", message: "Project must contain at least 1 screen form" });
  } else {
    const validCatalogTypes = new Set(WIDGET_CATALOG.map((c) => c.type));

    project.forms.forEach((form, fIdx) => {
      const fPath = `screens/${form.name || form.id || fIdx}`;
      if (!form.id || !form.name) {
        errors.push({
          severity: "error",
          path: `${fPath}`,
          message: "Form must have ID and Name",
          targetFormId: form.id,
        });
      }

      if (form.width <= 0 || form.height <= 0) {
        errors.push({
          severity: "error",
          path: `${fPath}.geometry`,
          message: `Form ${form.name} dimensions must be > 0`,
          targetFormId: form.id,
        });
      }

      const widgetIds = new Set<string>();
      (form.widgets ?? []).forEach((w, wIdx) => {
        const wPath = `${fPath}/${w.id || wIdx}`;

        if (!w.id || widgetIds.has(w.id)) {
          errors.push({
            severity: "error",
            path: `${wPath}.id`,
            message: `Duplicate or empty widget ID ${w.id} on screen ${form.name}`,
            targetFormId: form.id,
            targetWidgetId: w.id,
          });
        } else {
          widgetIds.add(w.id);
        }

        if (w.w <= 0 || w.h <= 0) {
          errors.push({
            severity: "error",
            path: `${wPath}.size`,
            message: `Widget ${w.id} size must be > 0 (w:${w.w}, h:${w.h})`,
            targetFormId: form.id,
            targetWidgetId: w.id,
          });
        }

        if (!validCatalogTypes.has(w.widget_type)) {
          errors.push({
            severity: "error",
            path: `${wPath}.type`,
            message: `Unknown widget type '${w.widget_type}' on widget ${w.id}`,
            targetFormId: form.id,
            targetWidgetId: w.id,
          });
        }
      });
    });
  }

  // 4. Scripts syntax check
  (project.tree ?? []).forEach((node) => {
    if (node.kind === "script" && node.content) {
      try {
        // Parsed by the same restricted grammar the runtime executes, so the
        // Designer cannot report a script as valid that the runtime rejects.
        parseScript(node.content);
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        errors.push({
          severity: "error",
          path: `scripts/${node.name}`,
          message: `Syntax error in script '${node.name}': ${msg}`,
          targetNodeId: node.id,
        });
      }
    }
  });

  return {
    valid: errors.length === 0,
    errors,
    warnings,
  };
}
