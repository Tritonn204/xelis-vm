use xelis_compiler::Compiler;
use xelis_environment::{Environment, EnvironmentError};
use xelis_builder::EnvironmentBuilder;
use xelis_lexer::Lexer;
use xelis_parser::Parser;
use xelis_types::{traits::{JSONHelper, Serializable}, Primitive};
use super::*;

#[track_caller]
fn prepare_module(code: &str) -> (Module, Environment) {
    prepare_module_with(code, EnvironmentBuilder::default())
}

#[track_caller]
fn prepare_module_with(code: &str, env: EnvironmentBuilder) -> (Module, Environment) {
    let tokens: Vec<_> = Lexer::new(code).into_iter().collect::<Result<_, _>>().unwrap();
    let (program, _) = Parser::with(tokens.into_iter(), &env).parse().unwrap();

    let env = env.build();
    let module = Compiler::new(&program, &env).compile().unwrap();

    (module, env)
}

#[track_caller]
fn try_run_code(code: &str, id: u16) -> Result<Primitive, VMError> {
    let (module, environment) = prepare_module(code);
    run_internal(module, &environment, id)
}

#[track_caller]
fn run_code_id(code: &str, id: u16) -> Primitive {
    try_run_code(code, id).unwrap()
}

#[track_caller]
fn run_code(code: &str) -> Primitive {
    run_code_id(code, 0)
}

#[track_caller]
fn test_code_expect_return(code: &str, expected: Primitive) {
    assert_eq!(
        run_code(code),
        expected
    );
}

#[track_caller]
fn test_code_id_expect_return(code: &str, expected: Primitive, id: u16) {
    assert_eq!(
        run_code_id(code, id),
        expected
    );
}

#[test]
fn test_max_gas() {
    let code = r#"
        entry main() {
            while true {}

            return 0
        }
    "#;

    let (module, environment) = prepare_module(code);
    let mut vm = VM::new(&module, &environment);
    vm.context_mut().set_gas_limit(1000);
    vm.invoke_entry_chunk(0).unwrap();

    assert!(matches!(vm.run(), Err(VMError::EnvironmentError(EnvironmentError::NotEnoughGas { .. }))));
}

#[test]
fn test_pow() {
    let code = r#"
        entry main() {
            return 2u64 ** 10
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(1024));
}

#[test]
fn test_pow_assign() {
    let code = r#"
        entry main() {
            let x: u64 = 2;
            x **= 10;
            return x
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(1024));
}

#[test]
fn test_u256() {
    let code = r#"
        entry main() {
            let x: u256 = 10;
            let y: u64 = 20;
            return (x + y as u256) as u64
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(30u32.into()));
}

#[test]
fn test_ternary_negative() {
    let code = r#"
        entry main() {
            let x: u64 = 20;
            let y: u64 = x == 10 ? 20 : 30;
            return y
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(30));
}

#[test]
fn test_and() {
    let code = r#"
        fn no_call() -> bool {
            return panic("no call")
        }

        entry main() {
            let x: u64 = 10;
            if (x != 10) && no_call() {
                panic("x is not 10")
            }
            return x
        }
    "#;

    assert_eq!(run_code_id(code, 1), Primitive::U64(10));
}

#[test]
fn test_and_positive() {
    let code = r#"
        fn test() -> bool {
            return true
        }

        entry main() {
            let x: u64 = 10;
            if (x == 10) && test() {
                return x
            }
            return panic("x is not 10")
        }
    "#;

    assert_eq!(run_code_id(code, 1), Primitive::U64(10));
}

#[test]
fn test_or() {
    let code = r#"
        fn no_call() -> bool {
            return panic("no call")
        }

        entry main() {
            let x: u64 = 10;
            if (x == 10) || no_call() {
                return 0
            }
            return panic("x is not 10")
        }
    "#;

    assert_eq!(run_code_id(code, 1), Primitive::U64(0));
}

#[test]
fn test_or_negative() {
    let code = r#"
        entry main() {
            let x: u64 = 10;
            if false || (x == 10) {
                return x
            }
            return panic("unexpected")
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(10));
}

#[test]
fn test_if_else() {
    let code = r#"
        entry main() {
            let x: u64 = 10;
            if x == 20 {
                x = 20
            } else {
                x = 30
            }
            return x
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(30));
}

#[test]
fn test_if_else_positive() {
    let code = r#"
        entry main() {
            let x: u64 = 10;
            if x == 10 {
                x = 20
            } else {
                x = 30
            }
            return x
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(20));
}

#[test]
fn test_if_else_if() {
    let code = r#"
        entry main() {
            let x: u64 = 10;
            if x == 20 {
                x = 20
            } else if x == 10 {
                x = 30
            } else {
                x = 40
            }
            return x
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(30));
}

#[test]
fn test_struct_access() {
    let code = r#"
        struct Test {
            x: u64,
            y: u64
        }

        entry main() {
            let t: Test = Test { x: 10, y: 20 };
            return t.x + t.y
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(30));
}

#[test]
fn test_struct_assign() {
    let code = r#"
        struct Test {
            x: u64,
            y: u64
        }

        entry main() {
            let t: Test = Test { x: 10, y: 20 };
            t.x = 30;
            return t.x + t.y
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(50));
}

#[test]
fn test_function_call() {
    let code = r#"
        fn add(a: u64, b: u64) -> u64 {
            return a + b;
        }

        entry main() {
            return add(10, 20);
        }
    "#;

    assert_eq!(run_code_id(code, 1), Primitive::U64(30));
}

#[test]
fn test_array_in_struct() {
    let code = r#"
        struct Test {
            arr: u64[]
        }

        entry main() {
            let t: Test = Test { arr: [10, 20, 30] };
            return t.arr[0] + t.arr[1] + t.arr[2]
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(60));
}

#[test]
fn test_nested_loops() {
    let code = r#"
        entry main() {
            let x: u64 = 0;
            for i: u64 = 0; i < 10; i += 1 {
                for j: u64 = 0; j < 10; j += 1 {
                    x = x + 1
                }
            }
            return x
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(100));
}

#[test]
fn test_for_array() {
    let code = r#"
        entry main() {
            let arr: u64[] = [10, 20, 30];
            let x: u64 = 0;
            for i: u32 = 0; i < arr.len(); i += 1 {
                let y: u64 = arr[i];
                x = x + y
            }
            return x
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(60));
}

#[test]
fn test_foreach_range() {
    let code = r#"
        entry main() {
            let x: u64 = 0;
            foreach i in 0..10 {
                x = x + i
            }
            return x
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(45));
}

#[test]
fn test_range_contains() {
    let code = r#"
        entry main() {
            let x: bool = (0..10).contains(5);
            return x as u64
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(1));
}


#[test]
fn test_range_contains_u256() {
    let code = r#"
        entry main() {
            let x: bool = (0u256..10u256).contains(5u256);
            return x as u64
        }
    "#;

    assert_eq!(run_code(code), Primitive::U64(1));
}

#[test]
fn test_range_collect() {
    let code = r#"
        entry main() {
            let x: u64[] = 0..10.collect();
            return x.len() as u64
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(10)
    );
}

#[test]
fn test_range_type() {
    let code = r#"
        entry main() {
            let x: range<u64> = 0..10;
            return x.count()
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(10)
    );
}

#[test]
fn test_stackoverflow() {
    let code = r#"
        entry main() {
            let x: u64 = 0;
            for i: u64 = 0; i < 1000000; i += 1 {
                x = x + 1
            }
            return x
        }"#;

    assert_eq!(run_code(code), Primitive::U64(1000000));

    let mut code = r#"
        entry main() {
            let a: u64 = 1;
            let b: u64 = a
    "#.to_string() + "+ a + a ".repeat(100000).as_str();
    code.push_str("; return b }");

    // TODO FIXME
    todo!("Fix stack overflow test");

    // assert_eq!(run_code(&code), Primitive::U64(10000 * 2 + 1));
}

#[test]
fn test_dangling_value_scoped() {
    let code = r#"
        entry main() {
            {
                10 + 5;
            }
            return 0
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(0)
    );
}

#[test]
fn test_dangling_value() {
    let code = r#"
        entry main() {
            10 + 5;
            return 0
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(0)
    );
}

#[test]
fn test_dangling_value_after_jump() {
    let code = r#"
        entry main() {
            if false {}
            10 + 5;
            return 0
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(0)
    );
}

#[test]
fn test_map() {
    let code = r#"
        entry main() {
            let x: map<string, u8> = {};
            x.insert("a", 10u8);
            let a: optional<u8> = x.get("a");
            return a.unwrap() as u64
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(10)
    );
}

#[test]
fn test_map_inline() {
    let code = r#"
        entry main() {
            return {
                "a": 10u8
            }.get("a").unwrap() as u64
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(10)
    );
}

#[test]
fn test_map_inline_with_vars() {
    let code = r#"
        entry main() {
            let a: string = "a";
            let b: u8 = 10;
            return {
                a: b
            }.get("a").unwrap() as u64
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(10)
    );
}

#[test]
fn test_self_reference_2d() {
    let code = r#"
        entry main() {
            let x: u64[][] = [[10]];
            x.push(x[0]);
            x[1][0] = 20;
            return x[0][0]
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(10)
    );
}

#[test]
fn test_self_reference_struct() {
    let code = r#"
        struct Test {
            x: u64
        }

        entry main() {
            let t: Test[] = [Test { x: 10 }];
            t.push(t[0]);
            t[1].x = 20;
            return t[0].x
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(10)
    );
}

#[test]
fn test_self_reference_map() {
    let code = r#"
        struct Dummy {
            x: u64
        }
        entry main() {
            let x: map<string, Dummy> = {};
            x.insert("a", Dummy { x: 10 });
            let dummy: Dummy = x.get("a").unwrap();
            x.insert("b", dummy);
            x.get("b").unwrap().x = 20;

            assert(!is_same_ptr(x.get("a").unwrap(), x.get("b").unwrap()));

            return x.get("a").unwrap().x
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(10)
    );
}

#[test]
fn test_enum() {
    let code = r#"
        enum Test {
            A,
            B { value: u64 }
        }

        entry main() {
            let x: Test = Test::B { value: 10 };
            return 10
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(10)
    );
}

#[test]
fn test_array_slice() {
    // Slice copy the array
    let code = r#"
        entry main() {
            let x: u64[] = [10, 20, 30, 40, 50];
            let y: u64[] = x.slice(1..4);

            assert(!is_same_ptr(y[0], x[1]));
            y.push(60);
            assert(!is_same_ptr(y[3], x[4]));

            y.push(x[4]);
            assert(!is_same_ptr(y[4], x[4]));

            return y[0] + y[1] + y[2]
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(90)
    );
}

#[test]
fn test_recursive_call() {
    let code = r#"
        fn fib(n: u64) -> u64 {
            if n == 0 {
                return 0
            } else if n == 1 {
                return 1
            }

            return fib(n - 1) + fib(n - 2)
        }

        entry main() {
            return fib(10)
        }
    "#;

    assert_eq!(
        run_code_id(code, 1),
        Primitive::U64(55)
    );
}

#[test]
fn test_const() {
    let code = r#"
        const X: u64 = 10;

        entry main() {
            return X
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(10)
    );
}

#[test]
fn test_const_add() {
    let code = r#"
        const ZERO: u64 = 0
        const HELLO_WORLD: string = "Hello World"

        entry main() {
            let message: string = HELLO_WORLD + " " + ZERO
            return message.len() as u64
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(13)
    );
}

#[test]
fn test_optional_cast() {
    let code = r#"
        struct Test { value: u64 }
        entry main() {
            let x: optional<Test> = Test { value: 10 };
            let y: optional<u64> = x.unwrap().value;
            let v: u64 = y.unwrap();
            return v
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(10)
    );
}


#[test]
fn test_optional_unwrap_or() {
    let code = r#"
        entry main() {
            let x: optional<u8> = null;
            return x.unwrap_or(10u8) as u64
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(10)
    );
}

#[test]
fn test_div_by_zero() {
    let code = r#"
        entry main() {
            let x: u64 = 10;
            let y: u64 = 0;
            return x / y
        }
    "#;

    assert!(
        matches!(
            try_run_code(code, 0),
            Err(VMError::DivisionByZero)
        )
    );
}

#[test]
fn test_path() {
    let code = r#"
        struct Value {
            value: string
        }

        fn (v Value) my_value() -> string {
            return v.value
        }

        struct Message {
            value: Value
        }

        fn (m Message) to_string() -> string {
            return m.value.value
        }

        entry main() {
            let message: Message = Message { value: Value { value: "Hello World!" } }
            assert(message.to_string() == "Hello World!")
            message.value.value += " from path"
            assert(message.to_string() == "Hello World! from path")
            return 0
        }
    "#;

    assert_eq!(
        run_code_id(code, 2),
        Primitive::U64(0)
    );
}

// #[track_caller]
// fn test_code_expect_return_with_env(code: &str, expected: u64, env: EnvironmentBuilder) {
//     assert_eq!(test_code_expect_value_with_env(env, &Signature::new("main".to_string(), None, Vec::new()), code).to_u64().unwrap(), expected);
// }

#[test]
fn test_self_assign() {
    // For mutability check, we must be sure to be able to use the same variable
    test_code_expect_return("entry main() { let a: u64 = 10; a = a; return a; }", Primitive::U64(10));
    test_code_expect_return("entry main() { let a: u64 = 10; a = a + a; return a; }", Primitive::U64(20));
}

#[test]
fn test_op_assignation() {
    test_code_expect_return("entry main() { let a: u64 = 10; a += 10; return a; }", Primitive::U64(20));
    test_code_expect_return("entry main() { let a: u64 = 10; a -= 10; return a; }", Primitive::U64(0));
    test_code_expect_return("entry main() { let a: u64 = 10; a *= 10; return a; }", Primitive::U64(100));
    test_code_expect_return("entry main() { let a: u64 = 10; a /= 10; return a; }", Primitive::U64(1));
    test_code_expect_return("entry main() { let a: u64 = 10; a %= 10; return a; }", Primitive::U64(0));

    // TODO: fix this, not sure why it's an outlier, parsing succeeds but VM fails. Could be bad error handling in the parser too
    test_code_expect_return("entry main() { let a: u64 = 10; a &= 10; return a; }", Primitive::U64(10));
    
    test_code_expect_return("entry main() { let a: u64 = 10; a |= 10; return a; }", Primitive::U64(10));
    test_code_expect_return("entry main() { let a: u64 = 10; a ^= 10; return a; }", Primitive::U64(0));
    test_code_expect_return("entry main() { let a: u64 = 10; a <<= 10; return a; }", Primitive::U64(10240));
    test_code_expect_return("entry main() { let a: u64 = 10; a >>= 10; return a; }", Primitive::U64(0));
}

#[test]
fn test_op_bool_assignation() {
    test_code_expect_return("entry main() { let a: bool = true; a = a && true; return a as u64; }", Primitive::U64(1));
    test_code_expect_return("entry main() { let a: bool = true; a = a && false; return a as u64; }", Primitive::U64(0));
    test_code_expect_return("entry main() { let a: bool = true; a = a || false; return a as u64; }", Primitive::U64(1));
    test_code_expect_return("entry main() { let a: bool = true; a = a || true; return a as u64; }", Primitive::U64(1));

    // TODO fix the 4 below, parsing succeeds but VM fails. Could be bad error handling in the parser too
    // |=
    test_code_expect_return("entry main() { let a: bool = false; a |= true; return a as u64; }", Primitive::U64(1));
    test_code_expect_return("entry main() { let a: bool = false; a |= false; return a as u64; }", Primitive::U64(0));
    // &=
    test_code_expect_return("entry main() { let a: bool = true; a &= true; return a as u64; }", Primitive::U64(1));
    test_code_expect_return("entry main() { let a: bool = true; a &= false; return a as u64; }", Primitive::U64(0));
}

#[test]
fn test_op_and() {
    // No call shouldn't be called
    let code = r#"
        fn no_call() -> bool { 
            return panic('should not call') 
        } 
        entry main() { 
            return (false && no_call()) as u64; 
        }
    "#;
    test_code_id_expect_return(code, Primitive::U64(0), 1);
    // Both should be called
    test_code_expect_return("entry main() { return (true && true) as u64; }", Primitive::U64(1));
    test_code_expect_return("entry main() { return (false && false) as u64; }", Primitive::U64(0));
}

#[test]
fn test_op_or() {
    // No call shouldn't be called
    let code = r#"
        fn no_call() -> bool { 
            return panic('should not call') 
        } 
        entry main() { 
            return (true || no_call()) as u64; 
        }
    "#;
    test_code_id_expect_return(code, Primitive::U64(1), 1);
    // Both are called
    test_code_expect_return("entry main() { return (false || true) as u64; }", Primitive::U64(1));
    // Both are called but none are true
    test_code_expect_return("entry main() { return (false || false) as u64; }", Primitive::U64(0));
}

#[test]
fn test_optional() {
    test_code_expect_return("entry main() { let a: u64[] = []; return a.first().unwrap_or(777); }", Primitive::U64(777));
}

#[test]
fn test_number_operations() {
    test_code_expect_return("entry main() { return 10; }", Primitive::U64(10));
    test_code_expect_return("entry main() { return 10 + 10; }", Primitive::U64(20));
    test_code_expect_return("entry main() { return 10 - 10; }", Primitive::U64(0));
    test_code_expect_return("entry main() { return 10 * 10; }", Primitive::U64(100));
    test_code_expect_return("entry main() { return 10 / 10; }", Primitive::U64(1));
    test_code_expect_return("entry main() { return 10 % 10; }", Primitive::U64(0));
    test_code_expect_return("entry main() { return (10 == 10) as u64; }", Primitive::U64(1));
    test_code_expect_return("entry main() { return (10 != 10) as u64; }", Primitive::U64(0));
    test_code_expect_return("entry main() { return (10 > 10) as u64; }", Primitive::U64(0));
    test_code_expect_return("entry main() { return (10 >= 10) as u64; }", Primitive::U64(1));
    test_code_expect_return("entry main() { return (10 < 10) as u64; }", Primitive::U64(0));
    test_code_expect_return("entry main() { return (10 <= 10) as u64; }", Primitive::U64(1));
    test_code_expect_return("entry main() { return 10 & 10; }", Primitive::U64(10));
    test_code_expect_return("entry main() { return 10 | 10; }", Primitive::U64(10));
    test_code_expect_return("entry main() { return 10 ^ 10; }", Primitive::U64(0));
    test_code_expect_return("entry main() { return 10 << 10; }", Primitive::U64(10240));
    test_code_expect_return("entry main() { return 10 >> 10; }", Primitive::U64(0));

    test_code_expect_return("entry main() { return 10 + 10 * 10; }", Primitive::U64(110));
    test_code_expect_return("entry main() { return (10 + 10) * 10; }", Primitive::U64(200));
}

#[test]
fn test_u128() {
    test_code_expect_return("entry main() { let j: u128 = 10; j = 2_u128 + j; return j as u64; }", Primitive::U64(12));
    test_code_expect_return("entry main() { let j: u128 = 10; j = ((2_u128 + j) * (3_u128 + j) * (4_u128 + j)); return j as u64; }", Primitive::U64(2184));
}

#[test]
fn test_array_all() {
    test_code_expect_return("entry main() { let a: u64[] = [1]; let b: u32 = 0; return a[b]; }", Primitive::U64(1));
    test_code_id_expect_return("fn test() -> u64[] { return [0, 1, 2]; } entry main() { let b: u32 = 0; return test()[b]; }", Primitive::U64(0), 1);

    test_code_expect_return("entry main() { let a: u64[] = [1, 2, 3]; return a[0]; }", Primitive::U64(1));
    test_code_expect_return("entry main() { let a: u64[] = [1, 2, 3]; return a[1]; }", Primitive::U64(2));
    test_code_expect_return("entry main() { let a: u64[] = [1, 2, 3]; return a[2]; }", Primitive::U64(3));

    test_code_expect_return("entry main() { let a: u64[] = [1, 2, 3]; return a[0] + a[1] + a[2]; }", Primitive::U64(6));
    test_code_expect_return("entry main() { let a: u64[] = [1, 2, 3]; a[0] = 10; return a[0]; }", Primitive::U64(10));
    test_code_expect_return("entry main() { let a: u64[] = [1, 2, 3]; a[0] = 10; return a[1]; }", Primitive::U64(2));
    test_code_expect_return("entry main() { let a: u64[] = [1, 2, 3]; a[0] = 10; return a[2]; }", Primitive::U64(3));
    test_code_expect_return("entry main() { let a: u64[] = [1, 2, 3]; a[0] = 10; return a[0] + a[1] + a[2]; }", Primitive::U64(15));

    // Push
    test_code_expect_return("entry main() { let a: u64[] = [1, 2, 3]; a.push(10); return a[3]; }", Primitive::U64(10));
    test_code_expect_return("entry main() { let a: u64[] = [1, 2, 3]; let v: u64 = 10; a.push(v); return a[0] + a[1] + a[2] + a[3]; }", Primitive::U64(16));
    test_code_expect_return("entry main() { let a: u64[] = [1, 2, 3]; let b: u64[] = []; let v: u64 = 10; b.push(10); a.push(b[0]); return a[0] + a[1] + a[2] + a[3]; }", Primitive::U64(16));

    // Pop
    test_code_expect_return("entry main() { let a: u64[] = [1, 2, 3]; a.pop(); return a.len() as u64; }", Primitive::U64(2));
}

#[test]
fn test_number_operations_priority() {
    test_code_expect_return("entry main() { return 10 + 10 * 10; }", Primitive::U64(110));
    test_code_expect_return("entry main() { return (10 + 10) * 10; }", Primitive::U64(200));

    test_code_expect_return("entry main() { return 10 + 10 / 5 + 3; }", Primitive::U64(15));
    test_code_expect_return("entry main() { return 10 + 10 / 5 * 3; }", Primitive::U64(16));
    test_code_expect_return("entry main() { return 10 + 10 / 5 + 3 * 10; }", Primitive::U64(42));
}

#[test]
fn test_basic_function_call() {
    test_code_id_expect_return("fn add(a: u64, b: u64) -> u64 { return a + b; } entry main() { return add(10, 10); }", Primitive::U64(20), 1);
    test_code_id_expect_return("fn add(a: u64, b: u64) -> u64 { return a + b; } entry main() { return add(10, add(10, 10)); }", Primitive::U64(30), 1);

    // With variable
    test_code_id_expect_return("fn add(a: u64, b: u64) -> u64 { return a + b; } entry main() { let a: u64 = 10; return add(a, 10); }", Primitive::U64(20), 1);
    test_code_id_expect_return("fn add(a: u64, b: u64) -> u64 { return a + b; } entry main() { let a: u64 = 10; return add(a, add(10, 10)); }", Primitive::U64(30), 1);
}

#[test]
fn test_function_call_on_value() {
    let code = r#"
        struct Test { a: u64 } 
        fn (v Test) add(b: u64) -> u64 { 
            return v.a + b; 
        } 
        entry main() { 
            let t: Test = Test {a: 10}; 
            return t.add(10); 
        }
    "#;
    test_code_id_expect_return(code, Primitive::U64(20), 1);

    let code = r#"
        struct Test { a: u64 } 
        fn (v Test) add(b: u64) -> u64 { 
            return v.a + b; 
        } 
        entry main() { 
            let t: Test = Test {a: 10}; 
            return t.add(t.add(10)); 
        }
    "#;
    test_code_id_expect_return(code, Primitive::U64(30), 1);

    let code = r#"
        struct Test { a: u64 } 
        fn (v Test) add(b: u64) { 
            v.a += b; 
        } 
        entry main() { 
            let t: Test = Test {a: 10}; 
            t.add(10); 
            return t.a 
        }
    "#;
    test_code_id_expect_return(code, Primitive::U64(20), 1);
}

#[test]
fn test_casting() {

    // Auto casting
    test_code_expect_return("fn main() -> u8 { return 10; }", Primitive::U8(10));
    test_code_expect_return("fn main() -> u16 { return 10; }", Primitive::U16(10));
    test_code_expect_return("fn main() -> u32 { return 10; }", Primitive::U32(10));
    test_code_expect_return("fn main() -> u64 { return 10; }", Primitive::U64(10));
    test_code_expect_return("fn main() -> u128 { return 10; }", Primitive::U128(10));

    // Explicit casting
    test_code_expect_return("fn main() -> u8 { let a: u64 = 10; return a as u8; }", Primitive::U8(10));
    test_code_expect_return("fn main() -> u16 { let a: u64 = 10; return a as u16; }", Primitive::U16(10));
    test_code_expect_return("fn main() -> u32 { let a: u64 = 10; return a as u32; }", Primitive::U32(10));
    test_code_expect_return("fn main() -> u64 { let a: u32 = 10; return a as u64; }", Primitive::U64(10));
    test_code_expect_return("fn main() -> u128 { let a: u64 = 10; return a as u128; }", Primitive::U128(10));

    let code = r#"
        fn add(left: u64, right: u64) -> u64 {
            return left + right;
        }

        entry main() {
            let a: u8 = 10;
            let b: u8 = 20;
            return add(a as u64, b as u64);
        }
    "#;
    test_code_id_expect_return(code, Primitive::U64(30), 1);

    let code = r#"entry main() {
        let a: u8 = 10;
        let b: u8 = 20;
        return a as u64 + b as u64;
    }"#;
    test_code_expect_return(code, Primitive::U64(30));
}

#[test]
fn test_string_number_concatenation() {
    test_code_expect_return("fn main() -> string { return (\"hello world\" + 10); }", Primitive::String("hello world10".to_string()));
    test_code_expect_return("fn main() -> string { return (10 + \"hello world\"); }", Primitive::String("10hello world".to_string()));
    test_code_expect_return("fn main() -> string { return (10 + \"hello world\" + 10); }", Primitive::String("10hello world10".to_string()));

    // With variables
    test_code_expect_return("fn main() -> string { let a: u64 = 10; return (\"hello world\" + a); }", Primitive::String("hello world10".to_string()));
    test_code_expect_return("fn main() -> string { let a: u64 = 10; return (a + \"hello world\"); }", Primitive::String("10hello world".to_string()));
    test_code_expect_return("fn main() -> string { let a: u64 = 10; return (a + \"hello world\" + a); }", Primitive::String("10hello world10".to_string()));
}

#[test]
fn test_negative_bool() {
    test_code_expect_return("fn main() -> bool { return !false; }", Primitive::Boolean(true));
    test_code_expect_return("fn main() -> bool { return !true; }", Primitive::Boolean(false));
    test_code_expect_return("fn main() -> bool { let add: bool = true; add = !add; return add; }", Primitive::Boolean(false));
}

#[test]
fn test_foreach() {
    test_code_expect_return("entry main() { let a: u64[] = [1, 2, 3]; let sum: u64 = 0; foreach i in a { sum += i; } return sum; }", Primitive::U64(6));
}

#[test]
fn test_while() {
    test_code_expect_return("entry main() { let a: u64 = 0; while a < 10 { a += 1; } return a; }", Primitive::U64(10));
}

#[test]
fn test_for() {
    test_code_expect_return("entry main() { let a: u64 = 1; for i: u64 = 0; i < 10; i += 1 { a *= 2; } return a; }", Primitive::U64(1024));
}

#[test]
fn test_break() {
    test_code_expect_return("entry main() { let a: u64 = 0; while a < 10 { a += 1; if a == 5 { break; } } return a; }", Primitive::U64(5));
}

#[test]
fn test_continue() {
    test_code_expect_return("entry main() { let i: u64 = 0; let a: u64 = 1; while i < 10 { i += 1; if i == 5 { continue; } a *= 2; } return a; }", Primitive::U64(512));
}

#[test]
fn test_string_equals() {
    test_code_expect_return("fn main() -> bool { return \"test\" == 'test'; }", Primitive::Boolean(true));
    test_code_expect_return("fn main() -> bool { return \"test\" == \"test2\"; }", Primitive::Boolean(false));
}

#[test]
fn test_ternary() {
    test_code_expect_return("entry main() { let a: u64 = 10; return a == 10 ? 0 : 1; }", Primitive::U64(0));
    test_code_expect_return("entry main() { let a: u64 = 0; return (a == 10) ? 1 : 0; }", Primitive::U64(0));
    test_code_expect_return("entry main() { let a: u64 = 0; let b: u64 = 0; a = (b == 0) ? 1 : b; return 0; }", Primitive::U64(0));
}

#[test]
fn test_if() {
    test_code_expect_return("entry main() { let a: u64 = 10; if a == 10 { return 0; } else { return 1; } }", Primitive::U64(0));
    test_code_expect_return("entry main() { let a: u64 = 10; if a == 0 { return 1; } else { return 0; } }", Primitive::U64(0));
}

#[test]
fn test_nested_if() {
    test_code_expect_return("entry main() { let a: u64 = 10; if a > 0 { if a == 10 { return 10; } else { return 0; } } else { return 0; } }", Primitive::U64(10));
    test_code_expect_return("entry main() { let a: u64 = 10; if a != 0 { if a == 10 { return 0; } else { return 11; } } return 999; }", Primitive::U64(0));
}

#[test]
fn test_else_if() {
    test_code_expect_return("entry main() { let a: u64 = 10; if a == 10 { return 10; } else if a == 0 { return 0; } else { return 1; } }", Primitive::U64(10));
    test_code_expect_return("entry main() { let a: u64 = 0; if a == 10 { return 10; } else if a == 0 { return 0; } else { return 1; } }", Primitive::U64(0));
    test_code_expect_return("entry main() { let a: u64 = 1; if a == 10 { return 10; } else if a == 0 { return 0; } else { return 1; } }", Primitive::U64(1));
}

// TODO: figure out how to do this without the interpreter crate
// #[test]
// fn test_struct_from_env() {
//     let mut env = EnvironmentBuilder::default();
//     env.register_structure("Test", vec![("a", Type::U64)]);
//     test_code_expect_return_with_env("entry main() { let t: Test = Test { a: 10 }; return t.a; }", 10, env);
// }

#[test]
fn test_struct() {
    test_code_expect_return("struct Test { a: u64 } entry main() { let t: Test = Test { a: 10 }; return t.a; }", Primitive::U64(10));
    test_code_expect_return("struct Test { a: u64 } entry main() { let t: Test = Test { a: 10 }; t.a = 20; return t.a; }", Primitive::U64(20));
}

#[test]
fn test_self_reference() {
    let code = r#"
        entry main() {
            let a: u8 = 100;
            let b: u64 = a.overflowing_add(a).unwrap() as u64;
            return b
        }
    "#;

    test_code_expect_return(code, Primitive::U64(200));
}

#[test]
fn test_self_reference_declared() {
    let code = r#"
        struct Test {
            a: u8
        }

        fn (t Test) overflowing_add(v: Test) -> u64 {
            return (t.a + v.a) as u64
        }

        entry main() {
            let t: Test = Test { a: 100 };
            return t.overflowing_add(t)
        }
    "#;

    test_code_id_expect_return(code, Primitive::U64(200), 1);
}

#[test]
fn test_function_nested() {
    let code = r#"
        fn add2(a: u32, b: u32) -> u32 {
            return a + b;
        }

        fn add(a: u32, b: u32) -> u32 {
            return add2(a, b);
        }

        entry main() {
            let c: u32 = add(1, 2);
            return 0;
        }
    "#;

    test_code_id_expect_return(code, Primitive::U64(0), 2);
}

#[test]
fn test_self_reference_owned_with_inner_ref() {
    let code = r#"
        struct Test {
            a: u8
        }

        fn (t Test) overflowing_add(v: Test) -> u64 {
            return t.a.overflowing_add(v.a).unwrap() as u64
        }

        entry main() {
            let b: u8[] = [100];
            let t: Test = Test { a: b[0] };
            t.a = 50;
            assert(b[0] == 100);
            return t.overflowing_add(t)
        }
    "#;

    test_code_id_expect_return(code, Primitive::U64(100), 1);
}

#[test]
fn test_optional_expect() {
    assert!(
        matches!(
            try_run_code("entry main() { let a: optional<u64> = null; return a.expect('a valid value'); }", 0),
            Err(VMError::EnvironmentError(EnvironmentError::Expect(_)))
        )
    );
}

#[test]
fn test_opaque_fn_call() {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct Foo;
    impl Serializable for Foo {}
    impl JSONHelper for Foo {}

    impl_opaque!("Foo", Foo);

    let mut env = EnvironmentBuilder::default();
    let ty = Type::Opaque(env.register_opaque::<Foo>("Foo"));

    env.register_native_function("foo", None, vec![], |_, _, _| {
        Ok(Some(Primitive::Opaque(Foo.into()).into()))
    }, 0, Some(ty.clone()));

    env.register_native_function("call", Some(ty), vec![], |_, _, _| {
        Ok(Some(Primitive::U64(0).into()))
    }, 0, Some(Type::U64));

    let code = r#"
        entry main() {
            let foo: Foo = foo();
            return foo.call()
        }
    "#;

    let (module, env) = prepare_module_with(code, env);

    assert_eq!(
        run_internal(module, &env, 0).unwrap(),
        Primitive::U64(0)
    );
}

#[test]
fn test_shadow_variable() {
    let code = r#"
        entry main() {
            let a: u64 = 10;
            let a: u64 = 20;
            return a
        }
    "#;

    assert_eq!(
        run_code(code),
        Primitive::U64(20)
    );
}

#[test]
fn test_null_as_return() {
    let code = r#"
        fn test() -> optional<u64> {
            return null
        }

        entry main() {
            let a: optional<u64> = test();
            return a.unwrap_or(10)
        }
    "#;

    assert_eq!(
        run_code_id(code, 1),
        Primitive::U64(10)
    );
}

#[test]
fn test_fn_call_with_optional_params() {
    let code = r#"
        fn test(a: optional<u64>) -> u64 {
            return a.unwrap_or(10)
        }

        entry main() {
            return test(5u64)
        }
    "#;

    assert_eq!(
        run_code_id(code, 1),
        Primitive::U64(5)
    );
}

#[test]
fn test_fn_params_immutable() {
    let code = r#"
        fn test(a: optional<u64>, v: string, arr: u64[]) {
            a = null;
            v = "zzzz";
            arr[0] = 1;
        }

        entry main() {
            let a: optional<u64> = 1000;
            let v: string = "aaa";
            let arr: u64[] = [0];

            test(a, v, arr);
            assert(v == "aaa");
            // Inner mutability is allowed
            assert(arr[0] == 1);

            return a.unwrap()
        }
    "#;

    assert_eq!(
        run_code_id(code, 1),
        Primitive::U64(1000)
    );
}

#[test]
fn test_types_compatibility() {
    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    struct DummyOpaque;

    impl JSONHelper for DummyOpaque {
        fn is_json_supported(&self) -> bool {
            false
        }

        fn serialize_json(&self) -> Result<serde_json::Value, anyhow::Error> {
            todo!()
        }
    }

    impl Serializable for DummyOpaque {
        fn get_size(&self) -> usize {
            0
        }

        fn is_serializable(&self) -> bool {
            false
        }

        fn serialize(&self, _: &mut Vec<u8>) -> usize {
            0
        }
    }

    impl_opaque!("Dummy", DummyOpaque);

    let mut env = EnvironmentBuilder::default();
    let ty  = Type::Opaque(env.register_opaque::<DummyOpaque>("Dummy"));
    env.register_native_function("test", None, vec![], |_, _, _| Ok(Some(ValueCell::Default(Primitive::Opaque(OpaqueWrapper::new(DummyOpaque))))), 0, Some(Type::Any)); 
    env.register_native_function("a", Some(ty), vec![], |_, _, _| Ok(Some(ValueCell::Default(Primitive::U64(0)))), 0, Some(Type::Any)); 
    env.register_static_function("static", Type::Bool, vec![], |_, _, _| Ok(Some(ValueCell::Default(Primitive::Null))), 0, Some(Type::Optional(Box::new(Type::Bool))));

    let (module, env) = prepare_module_with("
    struct Foo {
        dummy: optional<Dummy>
    }

    entry main() {
        let m: map<optional<optional<Foo>>, u64> = {
            null: 0
        };
        let _: map<optional<optional<Foo>>, u64> = {
            Foo {
                dummy: null
            }: 0
        };

        let _: u64 = (test() as Dummy).a();

        let foo: Foo = Foo {
            dummy: test()
        };
        let dummy: Dummy = test();
        foo.dummy = dummy;

        let _: optional<bool> = bool::static();

        return 0
    }", env);
    run_internal(module, &env, 0).unwrap();
}