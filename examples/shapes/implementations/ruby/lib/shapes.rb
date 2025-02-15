# frozen_string_literal: true

require_relative "shapes/version"

module Shapes
  class Shape
    def area
      raise NotImplementedError, "Subclass must implement abstract method"
    end

    def perimeter
      raise NotImplementedError, "Subclass must implement abstract method"
    end
  end

  class Polygon < Shape
    def edge_count
      raise NotImplementedError, "Subclass must implement abstract method"
    end

    def vertex_count
      raise NotImplementedError, "Subclass must implement abstract method"
    end
  end

  class Circle < Shape
    attr_reader :radius

    def initialize(radius)
      @radius = radius
    end

    def area
      Math::PI * @radius * @radius
    end

    def perimeter
      2 * Math::PI * @radius
    end

    def diameter
      2 * @radius
    end
  end

  class Rectangle < Polygon
    attr_reader :width, :height

    def initialize(width, height)
      @width = width
      @height = height
    end

    def area
      @width * @height
    end

    def perimeter
      2 * (@width + @height)
    end

    def edge_count
      4
    end

    def vertex_count
      4
    end

    def square?
      @width == @height
    end
  end

  class Triangle < Polygon
    attr_reader :side_a, :side_b, :side_c

    def initialize(side_a, side_b, side_c)
      @side_a = side_a
      @side_b = side_b
      @side_c = side_c
    end

    def area
      s = (@side_a + @side_b + @side_c) / 2.0
      Math.sqrt(s * (s - @side_a) * (s - @side_b) * (s - @side_c))
    end

    def perimeter
      @side_a + @side_b + @side_c
    end

    def edge_count
      3
    end

    def vertex_count
      3
    end
  end
end
