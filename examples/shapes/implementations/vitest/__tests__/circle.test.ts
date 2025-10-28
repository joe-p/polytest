import { expect, test, describe } from "vitest";
import { Circle } from "..";
describe("circle", () => {
  // Polytest Suite: circle

  const RADIUS = 7;
  const circle = new Circle(RADIUS);

  describe("circle", () => {
    // Polytest Group: circle

    test("non_numeric", () => {
      expect(() => {
        new Circle("Some radius");
      }).toThrow();
    });

    test("diameter", () => {
      expect(circle.diameter()).toBe(14);
    });

    test("radius", () => {
      expect(circle.radius).toBe(7);
    });
  });

  describe("shape", () => {
    // Polytest Group: shape

    test("perimeter", () => {
      expect(circle.perimeter()).toBe(Math.PI * RADIUS * 2);
    });

    test("area", () => {
      expect(circle.area()).toBe(Math.PI * RADIUS ** 2);
    });
  });
});
