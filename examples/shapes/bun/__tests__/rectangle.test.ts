import { expect, test, describe } from "bun:test";
import { Rectangle } from "..";

describe("rectangle", () => {
  // Polytest Suite: rectangle

  const rectangle = new Rectangle(10, 4);

  describe("rectangle", () => {
    // Polytest Group: rectangle

    test("is_square", () => {
      const square = new Rectangle(10, 10);
      expect(square.isSquare()).toBe(true);
      expect(rectangle.isSquare()).toBe(false);
    });
  });

  describe("polygon", () => {
    // Polytest Group: polygon

    test("edge_count", () => {
      expect(rectangle.edgeCount()).toBe(4);
    });

    test("vertex_count", () => {
      expect(rectangle.vertexCount()).toBe(4);
    });
  });

  describe("shape", () => {
    // Polytest Group: shape

    test("perimeter", () => {
      expect(rectangle.perimeter()).toBe(
        2 * (rectangle.width + rectangle.height),
      );
    });

    test("area", () => {
      expect(rectangle.area()).toBe(rectangle.width * rectangle.height);
    });
  });
});

