import assert from "node:assert/strict";
import { test, describe } from "node:test";
import { formatNumericValue, invokeWriteToTag, writeResultLabel } from "./config";

describe("formatNumericValue", () => {
  test("formats standard numbers with default parameters", () => {
    assert.strictEqual(formatNumericValue(5), "5");
    assert.strictEqual(formatNumericValue(123), "123");
  });

  test("pads leading zeros correctly for integers", () => {
    assert.strictEqual(formatNumericValue(5, 0, 4), "0005");
    assert.strictEqual(formatNumericValue(42, 0, 4), "0042");
    assert.strictEqual(formatNumericValue(12345, 0, 4), "12345");
  });

  test("handles decimals and padding together", () => {
    assert.strictEqual(formatNumericValue(5, 2, 4), "0005.00");
    assert.strictEqual(formatNumericValue(5.2, 2, 4), "0005.20");
    assert.strictEqual(formatNumericValue(12.345, 2, 3), "012.35");
  });

  test("handles negative numbers with padding", () => {
    assert.strictEqual(formatNumericValue(-5, 0, 4), "-0005");
    assert.strictEqual(formatNumericValue(-5.2, 2, 4), "-0005.20");
  });
});

describe("process write contract", () => {
  test("awaits the transport result instead of reporting success on invocation", async () => {
    let release!: () => void;
    const gate = new Promise<void>((resolve) => {
      release = resolve;
    });
    let settled = false;
    const pending = invokeWriteToTag("tag.command", false, async (tagId, value) => {
      await gate;
      return { status: "accepted", tagId, requestedValue: value };
    }, 7).then((result) => {
      settled = true;
      return result;
    });

    await Promise.resolve();
    assert.equal(settled, false);
    release();
    assert.deepEqual(await pending, {
      status: "accepted",
      tagId: "tag.command",
      requestedValue: 7,
    });
  });

  test("rejects unavailable, Designer and non-finite writes", async () => {
    await assert.rejects(() => invokeWriteToTag("", false, undefined, 1), /unavailable/);
    await assert.rejects(
      () =>
        invokeWriteToTag(
          "tag.command",
          true,
          async (tagId, value) => ({ status: "accepted", tagId, requestedValue: value }),
          1,
        ),
      /Designer/,
    );
    await assert.rejects(
      () =>
        invokeWriteToTag(
          "tag.command",
          false,
          async (tagId, value) => ({ status: "accepted", tagId, requestedValue: value }),
          Number.NaN,
        ),
      /finite/,
    );
  });

  test("labels observational and self-clearing receipts without claiming the requested state", () => {
    assert.equal(
      writeResultLabel({
        status: "observed",
        tagId: "cmd.reset",
        requestedValue: 1,
        observedValue: 0,
        verifyReadback: false,
        matches: false,
        selfCleared: true,
      }, "COMMAND"),
      "COMMAND OBSERVED 0 · SELF-CLEARED",
    );
  });
});
