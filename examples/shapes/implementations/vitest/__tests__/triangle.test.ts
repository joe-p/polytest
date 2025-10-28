import { expect, test, describe } from "vitest";
import { Triangle } from "..";
describe("triangle", () => {
  // Polytest Suite: triangle

  const triangle = new Triangle(3, 4, 5);

  describe("polygon", () => {
    // Polytest Group: polygon

    test("edge_count", () => {
      expect(triangle.edgeCount()).toBe(3);
    });

    test("vertex_count", () => {
      expect(triangle.vertexCount()).toBe(3);
    });
  });

  describe("shape", () => {
    // Polytest Group: shape

    test("perimeter", () => {
      expect(triangle.perimeter()).toBe(
        triangle.sideA + triangle.sideB + triangle.sideC,
      );
    });

    test("area", () => {
      const s = (triangle.sideA + triangle.sideB + triangle.sideC) / 2;

      expect(triangle.area()).toBe(
        Math.sqrt(
          s *
            (s - triangle.sideA) *
            (s - triangle.sideB) *
            (s - triangle.sideC),
        ),
      );
    });
  });
});
