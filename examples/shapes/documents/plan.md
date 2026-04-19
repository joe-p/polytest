# Polytest Test Plan

## Test Suites


### Circle

<table>
  <thead>
    <tr>
      <th>Name</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href="#shape">shape</a></td>
      <td>Tests that apply to any shape. A shape is a polygon OR a circle</td>
    </tr>
    <tr>
      <td><a href="#circle">circle</a></td>
      <td>Tests that only apply to a circle</td>
    </tr>
  </tbody>
</table>

### Rectangle

<table>
  <thead>
    <tr>
      <th>Name</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href="#polygon">polygon</a></td>
      <td>Tests that only apply to polygons</td>
    </tr>
    <tr>
      <td><a href="#rectangle">rectangle</a></td>
      <td>Tests that apply only to rectangles, a subset of polygons</td>
    </tr>
    <tr>
      <td><a href="#shape">shape</a></td>
      <td>Tests that apply to any shape. A shape is a polygon OR a circle</td>
    </tr>
  </tbody>
</table>

### Triangle

<table>
  <thead>
    <tr>
      <th>Name</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href="#polygon">polygon</a></td>
      <td>Tests that only apply to polygons</td>
    </tr>
    <tr>
      <td><a href="#shape">shape</a></td>
      <td>Tests that apply to any shape. A shape is a polygon OR a circle</td>
    </tr>
  </tbody>
</table>

### Extra

<table>
  <thead>
    <tr>
      <th>Name</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href="#extra">extra</a></td>
      <td>Extra tests not related to shapes but to test/show examples of extra functionality</td>
    </tr>
  </tbody>
</table>

## Test Groups


### Polygon

<table>
  <thead>
    <tr>
      <th>Name</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href="#vertex-count">vertex_count</a></td>
      <td>A polygon should accurately count the number of verticies it contains</td>
    </tr>
    <tr>
      <td><a href="#edge-count">edge_count</a></td>
      <td>A polygon should accurately count the number of edges it contains</td>
    </tr>
  </tbody>
</table>

### Rectangle

<table>
  <thead>
    <tr>
      <th>Name</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href="#is-square">is_square</a></td>
      <td>A rectangle should be able to accurately determine if it is a square</td>
    </tr>
  </tbody>
</table>

### Shape

<table>
  <thead>
    <tr>
      <th>Name</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href="#area">area</a></td>
      <td>A shape should be able to accurately calculate its area</td>
    </tr>
    <tr>
      <td><a href="#perimeter">perimeter</a></td>
      <td>A shape should be able to accurately calculate its perimeter (or circumference)</td>
    </tr>
  </tbody>
</table>

### Circle

<table>
  <thead>
    <tr>
      <th>Name</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href="#radius">radius</a></td>
      <td>A circle should be able to accurately calculate its radius</td>
    </tr>
    <tr>
      <td><a href="#diameter">diameter</a></td>
      <td>A circle should be able to accurately calculate its diameter</td>
    </tr>
    <tr>
      <td><a href="#non-numeric">non_numeric</a></td>
      <td>Using a non-numeric radius should throw an error</td>
    </tr>
  </tbody>
</table>

### Extra

<table>
  <thead>
    <tr>
      <th>Name</th>
      <th>Description</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href="#desc-new-line">desc_new_line</a></td>
      <td>Here<br/>Are<br/>New<br/>Lines</td>
    </tr>
  </tbody>
</table>

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

### desc_new_line

Here
Are
New
Lines
