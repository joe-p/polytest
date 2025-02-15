abstract class Shape {
  abstract area(): number;
  abstract perimeter(): number;
}

abstract class Polygon extends Shape {
  abstract edgeCount(): number;
  abstract vertexCount(): number;
}

export class Circle extends Shape {
  constructor(public radius: number) {
    super();
  }

  area(): number {
    return Math.PI * this.radius * this.radius;
  }

  perimeter(): number {
    return 2 * Math.PI * this.radius;
  }

  diameter(): number {
    return 2 * this.radius;
  }
}

export class Rectangle extends Polygon {
  constructor(
    public width: number,
    public height: number,
  ) {
    super();
  }

  area(): number {
    return this.width * this.height;
  }

  perimeter(): number {
    return 2 * (this.width + this.height);
  }

  edgeCount(): number {
    return 4;
  }

  vertexCount(): number {
    return 4;
  }

  isSquare(): boolean {
    return this.width === this.height;
  }
}

export class Triangle extends Polygon {
  constructor(
    public sideA: number,
    public sideB: number,
    public sideC: number,
  ) {
    super();
  }

  area(): number {
    const s = (this.sideA + this.sideB + this.sideC) / 2;

    return Math.sqrt(
      s * (s - this.sideA) * (s - this.sideB) * (s - this.sideC),
    );
  }

  perimeter(): number {
    return this.sideA + this.sideB + this.sideC;
  }

  edgeCount(): number {
    return 3;
  }

  vertexCount(): number {
    return 3;
  }
}

