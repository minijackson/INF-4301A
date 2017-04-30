use ast::*;
use ast::Expr::*;
use ast::Expr::{Array, Tuple};
use parser::parse_Expression;
use type_sys::Type;
use type_sys::Value::*;

#[test]
fn grouping() {
    let ast = Box::new(Grouping(Exprs {
                                    exprs: vec![Box::new(Value(Integer(1))),
                                                Box::new(Value(Integer(2)))],
                                }));

    assert_eq!(parse_Expression("(1, 2)").unwrap(), ast);

    let ast = Box::new(Grouping(Exprs { exprs: vec![Box::new(Value(Integer(1)))] }));

    // Single parenthesis is NOT a grouping
    assert_ne!(parse_Expression("(1)").unwrap(), ast);

    let ast = Box::new(Value(Integer(1)));

    // Still NOT a grouping
    assert_eq!(parse_Expression("(((((((((1)))))))))").unwrap(), ast);

    let ast = Box::new(Grouping(Exprs {
        exprs: vec![Box::new(Grouping(Exprs {
            exprs: vec![Box::new(Value(Integer(1))), Box::new(Value(Integer(2)))]
        })), Box::new(Value(Integer(3)))],
    }));

    assert_eq!(parse_Expression("((1, 2), 3)").unwrap(), ast);
}

#[test]
fn let_block() {
    let ast = Box::new(Let(vec![], vec![], Exprs { exprs: vec![] }));

    assert_eq!(parse_Expression("let in end").unwrap(), ast);

    let ast = Box::new(Let(vec![VariableDecl {
                                    name: "x".to_string(),
                                    value: Value(Integer(2)),
                                    span: Span(4, 14),
                                    value_span: Span(13, 14),
                                }],
                           vec![],
                           Exprs { exprs: vec![] }));

    assert_eq!(parse_Expression("let var x := 2 in end").unwrap(), ast);

    let ast = Box::new(Let(vec![VariableDecl {
                                    name: "x".to_string(),
                                    value: Value(Integer(2)),
                                    span: Span(4, 14),
                                    value_span: Span(13, 14),
                                },
                                VariableDecl {
                                    name: "y".to_string(),
                                    value: Value(Integer(42)),
                                    span: Span(15, 26),
                                    value_span: Span(24, 26),
                                }],
                           vec![],
                           Exprs { exprs: vec![] }));

    assert_eq!(parse_Expression("let var x := 2 var y := 42 in end").unwrap(),
               ast);

    let ast = Box::new(Let(vec![],
                           vec![FunctionDecl {
                                    name: "f".to_string(),
                                    args: vec![ArgumentDecl {
                                                   name: "x".to_string(),
                                                   type_: Type::Integer,
                                                   span: Span(15, 25),
                                               }],
                                    return_type: Type::Integer,
                                    signature_span: Span(4, 35),
                                    body: Box::new(Variable {
                                                       name: "x".to_string(),
                                                       span: Span(39, 40),
                                                   }),
                                    body_span: Span(39, 40),
                                }],
                           Exprs { exprs: vec![] }));

    assert_eq!(parse_Expression("let function f(x: Integer): Integer := x in end").unwrap(),
               ast);

    let ast = Box::new(Let(vec![VariableDecl {
                                    name: "y".to_string(),
                                    value: Value(Integer(2)),
                                    span: Span(4, 14),
                                    value_span: Span(13, 14),
                                }],
                           vec![FunctionDecl {
                                    name: "f".to_string(),
                                    args: vec![ArgumentDecl {
                                                   name: "x".to_string(),
                                                   type_: Type::Integer,
                                                   span: Span(26, 36),
                                               }],
                                    return_type: Type::Integer,
                                    signature_span: Span(15, 46),
                                    body: Box::new(BinaryOp {
                                                       lhs: Box::new(Variable {
                                                                         name: "x".to_string(),
                                                                         span: Span(50, 51),
                                                                     }),
                                                       rhs: Box::new(Variable {
                                                                         name: "y".to_string(),
                                                                         span: Span(54, 55),
                                                                     }),
                                                       op: BinaryOpCode::Add,
                                                       span: Span(50, 55),
                                                   }),
                                    body_span: Span(50, 55),
                                }],
                           Exprs {
                               exprs: vec![Box::new(Function {
                                                        name: "f".to_string(),
                                                        args: vec![(Box::new(Value(Integer(42))),
                                                                    Span(61, 63))],
                                                        span: Span(59, 64),
                                                    })],
                           }));

    assert_eq!(parse_Expression("let var y := 2 function f(x: Integer): Integer := x + y in f(42) end")
                   .unwrap(),
               ast);
}

#[test]
fn assign() {
    let ast = Box::new(Assign {
                           name: "x".to_string(),
                           name_span: Span(0, 1),
                           value: Box::new(Value(Integer(2))),
                           value_span: Span(5, 6),
                       });

    assert_eq!(parse_Expression("x := 2").unwrap(), ast);

    let ast = Box::new(Assign {
                           name: "x".to_string(),
                           name_span: Span(0, 1),
                           value: Box::new(Assign {
                                               name: "y".to_string(),
                                               name_span: Span(5, 6),
                                               value: Box::new(Value(Integer(2))),
                                               value_span: Span(10, 11),
                                           }),
                           value_span: Span(5, 11),
                       });

    assert_eq!(parse_Expression("x := y := 2").unwrap(), ast);

    assert!(parse_Expression("2 := 2").is_err());
    assert!(parse_Expression("x + 3 := 2").is_err());
}

#[test]
fn pattern_match() {
    let ast = Box::new(PatternMatch {
                           lhs: Box::new(Value(Integer(1))),
                           lhs_span: Span(6, 7),
                           rhs: Box::new(Value(Integer(1))),
                           rhs_span: Span(11, 12),
                       });

    assert_eq!(parse_Expression("match 1 := 1").unwrap(), ast);

    let ast = Box::new(PatternMatch {
                           lhs: Box::new(Array {
                                             values: vec![(Box::new(Tuple(vec![])), Span(7, 9))],
                                             declared_type: None,
                                             declared_type_span: None,
                                             span: Span(6, 10),
                                         }),
                           lhs_span: Span(6, 10),
                           rhs: Box::new(BinaryOp {
                                             lhs: Box::new(Value(Integer(2))),
                                             rhs: Box::new(Value(Integer(2))),
                                             op: BinaryOpCode::Add,
                                             span: Span(14, 17),
                                         }),
                           rhs_span: Span(14, 17),
                       });

    assert_eq!(parse_Expression("match [{}] := 2+2").unwrap(), ast);

    assert!(parse_Expression("match 2+2 := 2").is_err());
    assert!(parse_Expression("match [2+2] := 2").is_err());
    assert!(parse_Expression("match [{let in end}] := 2").is_err());
}

#[test]
fn function() {
    let ast = Box::new(Function {
                           name: "f".to_string(),
                           args: vec![],
                           span: Span(0, 3),
                       });

    assert_eq!(parse_Expression("f()").unwrap(), ast);

    let ast = Box::new(Function {
                           name: "f".to_string(),
                           args: vec![(Box::new(Value(Integer(42))), Span(2, 4))],
                           span: Span(0, 5),
                       });

    assert_eq!(parse_Expression("f(42)").unwrap(), ast);

    let ast = Box::new(Function {
                           name: "f".to_string(),
                           args: vec![(Box::new(BinaryOp {
                                                    lhs: Box::new(Value(Integer(2))),
                                                    rhs: Box::new(Value(Integer(2))),
                                                    op: BinaryOpCode::Add,
                                                    span: Span(2, 5),
                                                }),
                                       Span(2, 5)),
                                      (Box::new(BinaryOp {
                                                    lhs: Box::new(Value(Integer(2))),
                                                    rhs: Box::new(Value(Integer(2))),
                                                    op: BinaryOpCode::Add,
                                                    span: Span(6, 9),
                                                }),
                                       Span(6, 9))],
                           span: Span(0, 10),
                       });

    assert_eq!(parse_Expression("f(2+2,2+2)").unwrap(), ast);
}

#[test]
fn if_block() {
    let ast = Box::new(If {
                           cond: Box::new(Value(Bool(true))),
                           cond_span: Span(3, 7),
                           true_branch: Box::new(Value(Bool(true))),
                           true_branch_span: Span(13, 17),
                           false_branch: Box::new(Value(Bool(false))),
                           false_branch_span: Span(23, 28),
                       });

    assert_eq!(parse_Expression("if true then true else false").unwrap(),
               ast);

    let ast = Box::new(If {
                           cond: Box::new(If {
                                              cond: Box::new(Value(Bool(true))),
                                              cond_span: Span(6, 10),
                                              true_branch: Box::new(Value(Bool(true))),
                                              true_branch_span: Span(16, 20),
                                              false_branch: Box::new(Value(Bool(false))),
                                              false_branch_span: Span(26, 31),
                                          }),
                           cond_span: Span(3, 31),
                           true_branch: Box::new(If {
                                                     cond: Box::new(Value(Bool(true))),
                                                     cond_span: Span(40, 44),
                                                     true_branch: Box::new(Value(Bool(true))),
                                                     true_branch_span: Span(50, 54),
                                                     false_branch: Box::new(Value(Bool(false))),
                                                     false_branch_span: Span(60, 65),
                                                 }),
                           true_branch_span: Span(37, 65),
                           false_branch: Box::new(If {
                                                      cond: Box::new(Value(Bool(true))),
                                                      cond_span: Span(74, 78),
                                                      true_branch: Box::new(Value(Bool(true))),
                                                      true_branch_span: Span(84, 88),
                                                      false_branch: Box::new(Value(Bool(false))),
                                                      false_branch_span: Span(94, 99),
                                                  }),
                           false_branch_span: Span(71, 99),
                       });

    assert_eq!(parse_Expression("if if true then true else false then if true then true else false else if true then true else false").unwrap(), ast);
}

#[test]
fn while_block() {
    let ast = Box::new(While {
        cond: Box::new(Value(Bool(true))),
        cond_span: Span(6, 10),
        expr: Box::new(Value(Integer(42))),
    });

    assert_eq!(parse_Expression("while true do 42").unwrap(), ast);

    let ast = Box::new(While {
        cond: Box::new(While {
            cond: Box::new(Value(Bool(true))),
            cond_span: Span(12, 16),
            expr: Box::new(Value(Integer(42))),
        }),
        cond_span: Span(6, 22),
        expr: Box::new(While {
            cond: Box::new(Value(Bool(true))),
            cond_span: Span(32, 36),
            expr: Box::new(Value(Integer(42))),
        }),
    });


    assert_eq!(parse_Expression("while while true do 42 do while true do 42").unwrap(), ast);
}

#[test]
fn for_block() {
    let ast = Box::new(For {
        binding: Box::new(VariableDecl {
            name: "x".to_string(),
            span: Span(4, 14),
            value: Value(Integer(1)),
            value_span: Span(13, 14),
        }),
        goal: Box::new(Value(Integer(42))),
        goal_span: Span(18, 20),
        expr: Box::new(Variable {
            name: "x".to_string(),
            span: Span(24, 25),
        })
    });

    assert_eq!(parse_Expression("for var x := 1 to 42 do x").unwrap(), ast);

    let ast = Box::new(For {
        binding: Box::new(VariableDecl {
            name: "x".to_string(),
            span: Span(4, 16),
            value: BinaryOp {
                lhs: Box::new(Value(Integer(3))),
                rhs: Box::new(Value(Integer(2))),
                op: BinaryOpCode::Sub,
                span: Span(13, 16),
            },
            value_span: Span(13, 16),
        }),
        goal: Box::new(BinaryOp {
            lhs: Box::new(Value(Integer(6))),
            rhs: Box::new(Value(Integer(7))),
            op: BinaryOpCode::Mul,
            span: Span(20, 23),
        }),
        goal_span: Span(20, 23),
        expr: Box::new(Value(Integer(1))),
    });

    assert_eq!(parse_Expression("for var x := 3-2 to 6*7 do 1").unwrap(), ast);
}

#[test]
fn binary_operator() {
    let ast = Box::new(BinaryOp {
                           lhs: Box::new(Value(Integer(4))),
                           rhs: Box::new(Value(Integer(2))),
                           op: BinaryOpCode::Add,
                           span: Span(0, 3),
                       });

    assert_eq!(parse_Expression("4+2").unwrap(), ast);

    let ast = Box::new(BinaryOp {
                           lhs: Box::new(Value(Integer(4))),
                           rhs: Box::new(Value(Integer(2))),
                           op: BinaryOpCode::Sub,
                           span: Span(0, 3),
                       });

    assert_eq!(parse_Expression("4-2").unwrap(), ast);

    let ast = Box::new(BinaryOp {
                           lhs: Box::new(Value(Integer(4))),
                           rhs: Box::new(Value(Integer(2))),
                           op: BinaryOpCode::Mul,
                           span: Span(0, 5),
                       });

    assert_eq!(parse_Expression("4 * 2").unwrap(), ast);

    let ast = Box::new(BinaryOp {
                           lhs: Box::new(Value(Integer(4))),
                           rhs: Box::new(Value(Integer(2))),
                           op: BinaryOpCode::Eq,
                           span: Span(1, 4),
                       });

    assert_eq!(parse_Expression(" 4=2").unwrap(), ast);

    // Chaining

    let ast = Box::new(BinaryOp {
                           lhs: Box::new(BinaryOp {
                                             lhs: Box::new(Value(Integer(2))),
                                             rhs: Box::new(Value(Integer(3))),
                                             op: BinaryOpCode::Add,
                                             span: Span(0, 3),
                                         }),
                           rhs: Box::new(Value(Integer(4))),
                           op: BinaryOpCode::Add,
                           span: Span(0, 5),
                       });

    assert_eq!(parse_Expression("2+3+4").unwrap(), ast);

    let ast = Box::new(BinaryOp {
                           lhs: ast,
                           rhs: Box::new(Value(Integer(5))),
                           op: BinaryOpCode::Add,
                           span: Span(0, 9),
                       });

    assert_eq!(parse_Expression("2+3+4 + 5").unwrap(), ast);

    // Precedence

    let ast = Box::new(BinaryOp {
                           lhs: Box::new(BinaryOp {
                                             lhs: Box::new(Value(Integer(2))),
                                             rhs: Box::new(Value(Integer(2))),
                                             op: BinaryOpCode::Mul,
                                             span: Span(0, 3),
                                         }),
                           rhs: Box::new(Value(Integer(2))),
                           op: BinaryOpCode::Add,
                           span: Span(0, 5),
                       });

    assert_eq!(parse_Expression("2*2+2").unwrap(), ast);

    let ast = Box::new(BinaryOp {
                           lhs: Box::new(Value(Integer(2))),
                           rhs: Box::new(BinaryOp {
                                             lhs: Box::new(Value(Integer(2))),
                                             rhs: Box::new(Value(Integer(2))),
                                             op: BinaryOpCode::Mul,
                                             span: Span(2, 5),
                                         }),
                           op: BinaryOpCode::Add,
                           span: Span(0, 5),
                       });

    assert_eq!(parse_Expression("2+2*2").unwrap(), ast);

    let ast = Box::new(BinaryOp {
        op: BinaryOpCode::Eq,
        lhs: Box::new(BinaryOp {
            op: BinaryOpCode::Ne,
            lhs: Box::new(BinaryOp {
                op: BinaryOpCode::Add,
                lhs: Box::new(Value(Integer(2))),
                rhs: Box::new(BinaryOp {
                    op: BinaryOpCode::Div,
                    lhs: Box::new(BinaryOp {
                        op: BinaryOpCode::Mul,
                        lhs: Box::new(Value(Integer(2))),
                        rhs: Box::new(Value(Integer(2))),
                        span: Span(2, 5),
                    }),
                    rhs: Box::new(Value(Integer(2))),
                    span: Span(2, 7),
                }),
                span: Span(0, 7),
            }),
            rhs: Box::new(BinaryOp {
                op: BinaryOpCode::Ge,
                lhs: Box::new(BinaryOp {
                    op: BinaryOpCode::Lt,
                    lhs: Box::new(Value(Integer(2))),
                    rhs: Box::new(Value(Integer(2))),
                    span: Span(11, 14),
                }),
                rhs: Box::new(Value(Integer(2))),
                span: Span(11, 17),
            }),
            span: Span(0, 17),
        }),
        rhs: Box::new(Value(Integer(2))),
        span: Span(0, 21),
    });

    assert_eq!(parse_Expression("2+2*2/2 <> 2<2>=2 = 2").unwrap(), ast);
}

#[test]
fn unary_operator() {
    let ast = Box::new(UnaryOp {
        expr: Box::new(Value(Integer(2))),
        op: UnaryOpCode::Minus,
        span: Span(0, 2),
    });

    assert_eq!(parse_Expression("-2").unwrap(), ast);

    let ast = Box::new(UnaryOp {
        expr: Box::new(Value(Integer(2))),
        op: UnaryOpCode::Plus,
        span: Span(0, 2),
    });

    assert_eq!(parse_Expression("+2").unwrap(), ast);

    let ast = Box::new(BinaryOp {
        lhs: Box::new(BinaryOp {
            lhs: Box::new(Value(Integer(2))),
            rhs: Box::new(UnaryOp {
                expr: Box::new(Value(Integer(2))),
                op: UnaryOpCode::Plus,
                span: Span(2, 4),
            }),
            op: BinaryOpCode::Add,
            span: Span(0, 4),
        }),
        rhs: Box::new(Value(Integer(2))),
        op: BinaryOpCode::Add,
        span: Span(0, 6),
    });

    assert_eq!(parse_Expression("2++2+2").unwrap(), ast);

    let ast = Box::new(BinaryOp {
        lhs: Box::new(BinaryOp {
            lhs: Box::new(Value(Integer(2))),
            rhs: Box::new(UnaryOp {
                expr: Box::new(Value(Integer(2))),
                op: UnaryOpCode::Minus,
                span: Span(2, 4),
            }),
            op: BinaryOpCode::Add,
            span: Span(0, 4),
        }),
        rhs: Box::new(Value(Integer(2))),
        op: BinaryOpCode::Add,
        span: Span(0, 6),
    });

    assert_eq!(parse_Expression("2+-2+2").unwrap(), ast);
}

#[test]
fn cast() {
    let ast = Box::new(Cast {
        expr: Box::new(Value(Integer(42))),
        expr_span: Span(0, 2),
        dest: Type::Float
    });

    assert_eq!(parse_Expression("42 as Float").unwrap(), ast);

    let ast = Box::new(BinaryOp {
        lhs: Box::new(Value(Integer(2))),
        rhs: Box::new(Cast {
            expr: Box::new(Value(Integer(42))),
            expr_span: Span(4, 6),
            dest: Type::Float
        }),
        op: BinaryOpCode::Add,
        span: Span(0, 15),
    });

    assert_eq!(parse_Expression("2 + 42 as Float").unwrap(), ast);

    let ast = Box::new(Cast {
        expr: Box::new(BinaryOp {
            lhs: Box::new(Value(Integer(2))),
            rhs: Box::new(Value(Integer(42))),
            op: BinaryOpCode::Add,
            span: Span(1, 7),
        }),
        expr_span: Span(0, 8),
        dest: Type::Float
    });

    assert_eq!(parse_Expression("(2 + 42) as Float").unwrap(), ast);

    let ast = Box::new(Cast {
        expr: Box::new(Value(Integer(1))),
        expr_span: Span(0, 1),
        dest: Type::Array(Box::new(Type::Array(Box::new(Type::Integer)))),
    });

    assert_eq!(parse_Expression("1 as Array(Array(Integer))").unwrap(), ast);

    let ast = Box::new(Cast {
        expr: Box::new(Value(Integer(1))),
        expr_span: Span(0, 1),
        dest: Type::Tuple(vec![Type::Array(Box::new(Type::Integer)), Type::Tuple(vec![Type::Integer, Type::Bool])]),
    });

    assert_eq!(parse_Expression("1 as Tuple(Array(Integer), Tuple(Integer, Bool))").unwrap(), ast);
}

#[test]
fn variable() {
    let ast = Box::new(Variable {
        name: "x".to_string(),
        span: Span(0, 1),
    });

    assert_eq!(parse_Expression("x").unwrap(), ast);

    let ast = Box::new(Variable {
        name: "x_y".to_string(),
        span: Span(0, 3),
    });

    assert_eq!(parse_Expression("x_y").unwrap(), ast);

    assert!(parse_Expression("2x").is_err());
}

#[test]
fn array() {
    let ast = Box::new(Array {
        values: vec![],
        declared_type: None,
        declared_type_span: None,
        span: Span(0, 2),
    });

    assert_eq!(parse_Expression("[]").unwrap(), ast);

    let ast = Box::new(Array {
        values: vec![(Box::new(Value(Integer(1))), Span(1, 2)), (Box::new(Value(Integer(2))), Span(4, 5))],
        declared_type: None,
        declared_type_span: None,
        span: Span(0, 6),
    });

    assert_eq!(parse_Expression("[1, 2]").unwrap(), ast);

    let ast = Box::new(Array {
        values: vec![(Box::new(Value(Integer(1))), Span(8, 9)), (Box::new(Value(Integer(2))), Span(11, 12))],
        declared_type: Some(Type::Integer),
        declared_type_span: Some(Span(0, 7)),
        span: Span(0, 13),
    });

    assert_eq!(parse_Expression("Integer[1, 2]").unwrap(), ast);

    let ast = Box::new(Array {
        values: vec![(Box::new(Array {
            values: vec![],
            declared_type: None,
            declared_type_span: None,
            span: Span(1, 3),
        }), Span(1, 3))],
        declared_type: None,
        declared_type_span: None,
        span: Span(0, 4),
    });

    assert_eq!(parse_Expression("[[]]").unwrap(), ast);
}

#[test]
fn tuple() {
    let ast = Box::new(Tuple(vec![]));

    assert_eq!(parse_Expression("{}").unwrap(), ast);

    let ast = Box::new(Tuple(vec![Box::new(Value(Integer(42))), Box::new(Value(Integer(69)))]));

    assert_eq!(parse_Expression("{42, 69}").unwrap(), ast);

    let ast = Box::new(Tuple(vec![Box::new(BinaryOp {
        lhs: Box::new(Value(Integer(2))),
        rhs: Box::new(Value(Integer(2))),
        span: Span(1, 6),
        op: BinaryOpCode::Add,
    })]));

    assert_eq!(parse_Expression("{2 + 2}").unwrap(), ast);

    let ast = Box::new(Tuple(vec![Box::new(Tuple(vec![Box::new(Value(Integer(2)))])), Box::new(Value(Bool(false)))]));

    assert_eq!(parse_Expression("{{2}, false}").unwrap(), ast);
}

#[test]
fn value() {
    let ast = Box::new(Value(Integer(42)));
    assert_eq!(parse_Expression("42").unwrap(), ast);

    // Does not fit in a 64 bit integer type
    assert!(parse_Expression("10000000000000000000000000000000").is_err());

    let ast = Box::new(Value(Float(13.37)));
    assert_eq!(parse_Expression("13.37").unwrap(), ast);

    let ast = Box::new(Value(Float(69f64)));
    assert_eq!(parse_Expression("69.").unwrap(), ast);

    let ast = Box::new(Value(Bool(true)));
    assert_eq!(parse_Expression("true").unwrap(), ast);

    let ast = Box::new(Value(Bool(false)));
    assert_eq!(parse_Expression("false").unwrap(), ast);

    let ast = Box::new(Value(Str("hello".to_string())));
    assert_eq!(parse_Expression(r#""hello""#).unwrap(), ast);

    let ast = Box::new(Value(Str(r#"hel"lo"#.to_string())));
    assert_eq!(parse_Expression(r#""hel\"lo""#).unwrap(), ast);

    let ast = Box::new(Value(Str(r#"hel"lo"#.to_string())));
    assert_eq!(parse_Expression(r#""hel\"lo""#).unwrap(), ast);

    let ast = Box::new(Value(Str("hello".to_string())));
    assert_eq!(parse_Expression(r#""hel\x6co""#).unwrap(), ast);

    let ast = Box::new(Value(Str("hello".to_string())));
    assert_eq!(parse_Expression(r#""hel\x6Co""#).unwrap(), ast);

    assert!(parse_Expression(r#""hel\x""#).is_err());

    let ast = Box::new(Value(Str("hello".to_string())));
    assert_eq!(parse_Expression(r#""hel\u006co""#).unwrap(), ast);

    let ast = Box::new(Value(Str("hello".to_string())));
    assert_eq!(parse_Expression(r#""hel\u006Co""#).unwrap(), ast);

    let ast = Box::new(Value(Str("I ♥ Rust".to_string())));
    assert_eq!(parse_Expression(r#""I \u2665 Rust""#).unwrap(), ast);

    assert!(parse_Expression(r#""hel\u""#).is_err());
}
