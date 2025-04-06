# frozen_string_literal: true

require "test_helper"

# circle
class TestCircle < Minitest::Test
  def setup
    @circle = Circle.new(7)
  end

  # Polytest Suite: circle

  # Polytest Group: circle

  def test_non_numeric
    assert_raises ArgumentError do
      Circle.new("Some radius")
    end
  end

  def test_diameter
    assert_equal 14, @circle.diameter
  end

  def test_radius
    assert_equal 7, @circle.radius
  end

  # Polytest Group: shape

  def test_perimeter
    assert_equal Math::PI * @circle.radius * 2, @circle.perimeter
  end

  def test_area
    assert_equal Math::PI * @circle.radius**2, @circle.area
  end
end
