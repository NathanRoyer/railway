use crate::*;

fn serialize_deserialize(p: &Program) -> Option<Program> {
    let sz = p.file_size();
    let mut buffer = Vec::with_capacity(sz);
    match sz == p.dump(&mut buffer).unwrap() {
        true => Some(Program::parse(&buffer).unwrap()),
        false => None,
    }
}

#[test]
fn partial_eq() {
    // todo: use existing, bigger programs
    assert_eq!(Program::new(), Program::new());
}

#[test]
fn round_trip() {
    let a = Program::new();
    assert_eq!(a, serialize_deserialize(&a).unwrap());
}

#[test]
fn arguments() {
    let mut a = Program::new();
    a.arguments.push(Argument {
        name: None,
        value: Couple::new(9999999.0, -1.0 / 3.0),
        range: (Couple::new(0.0, -10.0), Couple::new(10000000.0, 0.0)),
    });
    a.arguments.push(Argument {
        name: Some("Shortie No Mass".into()),
        value: C_ZERO,
        range: (C_ZERO, C_ZERO),
    });
    assert_eq!(a, serialize_deserialize(&a).unwrap());
}

#[test]
fn instructions() {
    let mut a = Program::new();
    let c = Couple::new(5.0, 10.0);
    a.arguments.push(Argument {
        name: None,
        value: c,
        range: (c, c),
    });
    a.instructions.push(Instruction {
        operation: Operation::Multiply2,
        operands: [0, 0, 0],
    });
    a.instructions.push(Instruction {
        operation: Operation::Add2,
        operands: [1, 0, 0],
    });
    assert_eq!(a, serialize_deserialize(&a).unwrap());

    let mut stack = a.create_stack();
    a.compute(&mut stack);
    assert_eq!(stack[2], Couple::new(30.0, 110.0));
}

#[test]
fn primitives() {
    let mut a = Program::new();
    for _ in 0..12 {
        a.arguments.push(Argument {
            name: None,
            value: C_ZERO,
            range: (C_ZERO, C_ZERO),
        });
    }
    a.arcs.push(Arc {
        center: 0,
        angular_range: 1,
        radii: 2,
    });
    a.cubic_curves.push(CubicCurve {
        points: [3, 4, 5, 6],
    });
    a.quadratic_curves
        .push(QuadraticCurve { points: [7, 8, 9] });
    a.lines.push(Line { points: [10, 11] });
    assert_eq!(a, serialize_deserialize(&a).unwrap());
}
