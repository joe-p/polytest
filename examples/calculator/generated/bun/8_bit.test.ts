import { expect, test, describe } from "bun:test";

const MAX_VALUE = 2 ** 8 - 1;

function add(a: number, b: number): number {
  return (a + b) % (MAX_VALUE + 1);
}

function multiply(a: number, b: number): number {
  return (a * b) % (MAX_VALUE + 1);
}

describe("8bit", () => {
  // Polytest Suite: 8bit

  describe("multiply", () => {
    // Polytest Group: multiply

    test("product_overflow", () => {
      expect(multiply(MAX_VALUE, 2)).toBe(MAX_VALUE - 1);
    });

    test("product", () => {
      // Intentional fail
      expect(multiply(2, 3)).toBe(5);
    });
  });

  describe("add", () => {
    // Polytest Group: add

    test("overflow_sum", () => {
      expect(add(MAX_VALUE, 1)).toBe(0);
    });

    test("sum", () => {
      expect(add(2, 3)).toBe(5);
    });
  });
});
