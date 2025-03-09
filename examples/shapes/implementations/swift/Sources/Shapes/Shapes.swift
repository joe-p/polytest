// The Swift Programming Language
// https://docs.swift.org/swift-book

import Foundation

protocol Shape {
    func area() -> Double
    func perimeter() -> Double
}

protocol Polygon: Shape {
    func edgeCount() -> Int
    func vertexCount() -> Int
}

public class Circle: Shape {
    public let radius: Double

    public init(radius: Double) {
        self.radius = radius
    }

    public func area() -> Double {
        return Double.pi * radius * radius
    }

    public func perimeter() -> Double {
        return 2 * Double.pi * radius
    }

    public func diameter() -> Double {
        return 2 * radius
    }
}

public class Rectangle: Polygon {
    public let width: Double
    public let height: Double

    public init(width: Double, height: Double) {
        self.width = width
        self.height = height
    }

    public func area() -> Double {
        return width * height
    }

    public func perimeter() -> Double {
        return 2 * (width + height)
    }

    public func edgeCount() -> Int {
        return 4
    }

    public func vertexCount() -> Int {
        return 4
    }

    public func isSquare() -> Bool {
        return width == height
    }
}

public class Triangle: Polygon {
    public let sideA: Double
    public let sideB: Double
    public let sideC: Double

    public init(sideA: Double, sideB: Double, sideC: Double) {
        self.sideA = sideA
        self.sideB = sideB
        self.sideC = sideC
    }

    public func area() -> Double {
        let s = (sideA + sideB + sideC) / 2
        return sqrt(s * (s - sideA) * (s - sideB) * (s - sideC))
    }

    public func perimeter() -> Double {
        return sideA + sideB + sideC
    }

    public func edgeCount() -> Int {
        return 3
    }

    public func vertexCount() -> Int {
        return 3
    }
}
