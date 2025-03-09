import Foundation
import Testing

@testable import Shapes

// Polytest Suite: rectangle

// Polytest Group: shape

@Test("rectangle: perimeter")
func rectanglePerimeter() throws {
    let rectangle = Rectangle(width: 10, height: 4)

    #expect(rectangle.perimeter() == 2 * (rectangle.width + rectangle.height))
}

@Test("rectangle: area")
func rectangleArea() throws {
    let rectangle = Rectangle(width: 10, height: 4)

    #expect(rectangle.area() == rectangle.width * rectangle.height)
}

// Polytest Group: rectangle

@Test("rectangle: is_square")
func rectangleIsSquare() throws {
    let rectangle = Rectangle(width: 10, height: 4)
    let square = Rectangle(width: 10, height: 10)

    #expect(square.isSquare() == true)
    #expect(rectangle.isSquare() == false)
}

// Polytest Group: polygon

@Test("rectangle: edge_count")
func rectangleEdgeCount() throws {
    let rectangle = Rectangle(width: 10, height: 4)

    #expect(rectangle.edgeCount() == 4)
}

@Test("rectangle: vertex_count")
func rectangleVertexCount() throws {
    let rectangle = Rectangle(width: 10, height: 4)

    #expect(rectangle.vertexCount() == 4)
}
