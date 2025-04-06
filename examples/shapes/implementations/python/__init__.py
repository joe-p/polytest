from abc import ABC, abstractmethod
import math


class Shape(ABC):
    @abstractmethod
    def area(self) -> float:
        pass

    @abstractmethod
    def perimeter(self) -> float:
        pass


class Polygon(Shape):
    @abstractmethod
    def edge_count(self) -> int:
        pass

    @abstractmethod
    def vertex_count(self) -> int:
        pass


class Circle(Shape):
    def __init__(self, radius: float):
        if not isinstance(radius, (int, float, complex)):
            raise ValueError(f"radius must be a number, got {type(radius)}")

        self.radius = radius

    def area(self) -> float:
        return math.pi * self.radius * self.radius

    def perimeter(self) -> float:
        return 2 * math.pi * self.radius

    def diameter(self) -> float:
        return 2 * self.radius


class Rectangle(Polygon):
    def __init__(self, width: float, height: float):
        self.width = width
        self.height = height

    def area(self) -> float:
        return self.width * self.height

    def perimeter(self) -> float:
        return 2 * (self.width + self.height)

    def edge_count(self) -> int:
        return 4

    def vertex_count(self) -> int:
        return 4

    def is_square(self) -> bool:
        return self.width == self.height


class Triangle(Polygon):
    def __init__(self, side_a: float, side_b: float, side_c: float):
        self.side_a = side_a
        self.side_b = side_b
        self.side_c = side_c

    def area(self) -> float:
        s = (self.side_a + self.side_b + self.side_c) / 2
        return math.sqrt(s * (s - self.side_a) * (s - self.side_b) * (s - self.side_c))

    def perimeter(self) -> float:
        return self.side_a + self.side_b + self.side_c

    def edge_count(self) -> int:
        return 3

    def vertex_count(self) -> int:
        return 3
