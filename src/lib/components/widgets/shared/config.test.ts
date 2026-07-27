import assert from "node:assert/strict";
import { test, describe } from "node:test";
import { formatNumericValue } from "./config";

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
