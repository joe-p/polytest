import Testing
import Foundation

@testable import Shapes

// Polytest Suite: circle

// Polytest Group: circle

@Test("circle: diameter") 
func circleDiameter() throws {
    let radius = 7.0
    let circle = Circle(radius: radius)
    
    #expect(circle.diameter() == 14.0)
}

@Test("circle: radius") 
func circleRadius() throws {
    let radius = 7.0
    let circle = Circle(radius: radius)
    
    #expect(circle.radius == 7.0)
}

// Polytest Group: shape

@Test("circle: perimeter") 
func circlePerimeter() throws {
    let radius = 7.0
    let circle = Circle(radius: radius)
    
    #expect(circle.perimeter() == Double.pi * radius * 2)
}

@Test("circle: area") 
func circleArea() throws {
    let radius = 7.0
    let circle = Circle(radius: radius)
    
    #expect(circle.area() == Double.pi * radius * radius)
}