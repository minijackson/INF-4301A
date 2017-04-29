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

//#[test]
//fn for_block() {
    //let ast = Box::new(For {

    //});

    //assert_eq!(parse_Expression("for var x := 1 to 42 do x").unwrap(), ast);
//}

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
