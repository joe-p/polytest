# Polytest Test Plan
## Test Suites

### Circle

| Name | Description |
| --- | --- |
| [shape](#shape) | Tests that apply to any shape. A shape is a polygon OR a circle |
| [circle](#circle) | Tests that only apply to a circle |

### Rectangle

| Name | Description |
| --- | --- |
| [polygon](#polygon) | Tests that only apply to polygons |
| [rectangle](#rectangle) | Tests that apply only to rectangles, a subset of polygons |
| [shape](#shape) | Tests that apply to any shape. A shape is a polygon OR a circle |

### Triangle

| Name | Description |
| --- | --- |
| [polygon](#polygon) | Tests that only apply to polygons |
| [shape](#shape) | Tests that apply to any shape. A shape is a polygon OR a circle |

## Test Groups

### Polygon

| Name | Description |
| --- | --- |
| [vertex_count](#vertex-count) | A polygon should accurately count the number of verticies it contains |
| [edge_count](#edge-count) | A polygon should accurately count the number of edges it contains |

### Rectangle

| Name | Description |
| --- | --- |
| [is_square](#is-square) | A rectangle should be able to accurately determine if it is a square |

### Shape

| Name | Description |
| --- | --- |
| [area](#area) | A shape should be able to accurately calculate its area |
| [perimeter](#perimeter) | A shape should be able to accurately calculate its perimeter (or circumference) |

### Circle

| Name | Description |
| --- | --- |
| [radius](#radius) | A circle should be able to accurately calculate its radius |
| [diameter](#diameter) | A circle should be able to accurately calculate its diameter |
| [non_numeric](#non-numeric) | Using a non-numeric radius should throw an error |

## Test Cases

### vertex_count

A polygon should accurately count the number of verticies it contains

### edge_count

A polygon should accurately count the number of edges it contains

### is_square

A rectangle should be able to accurately determine if it is a square

### area

A shape should be able to accurately calculate its area

### perimeter

A shape should be able to accurately calculate its perimeter (or circumference)

### radius

A circle should be able to accurately calculate its radius

### diameter

A circle should be able to accurately calculate its diameter

### non_numeric

Using a non-numeric radius should throw an error
