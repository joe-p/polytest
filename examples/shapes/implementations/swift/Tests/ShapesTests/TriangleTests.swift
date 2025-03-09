import Testing
import Foundation

@testable import Shapes

// Polytest Suite: triangle

// Polytest Group: shape

@Test("triangle: perimeter")
func trianglePerimeter() throws {
    let triangle = Triangle(sideA: 3, sideB: 4, sideC: 5)

    #expect(triangle.perimeter() == triangle.sideA + triangle.sideB + triangle.sideC)
}

@Test("triangle: area")
func triangleArea() throws {
    let triangle = Triangle(sideA: 3, sideB: 4, sideC: 5)

    let s = (triangle.sideA + triangle.sideB + triangle.sideC) / 2
    let expectedArea = sqrt(s * (s - triangle.sideA) * (s - triangle.sideB) * (s - triangle.sideC))

    #expect(triangle.area() == expectedArea)
}

// Polytest Group: polygon

@Test("triangle: edge_count")
func triangleEdgeCount() throws {
    let triangle = Triangle(sideA: 3, sideB: 4, sideC: 5)

    #expect(triangle.edgeCount() == 3)
}

@Test("triangle: vertex_count")
func triangleVertexCount() throws {
    let triangle = Triangle(sideA: 3, sideB: 4, sideC: 5)

    #expect(triangle.vertexCount() == 3)
}
