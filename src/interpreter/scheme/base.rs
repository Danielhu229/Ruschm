use crate::environment::IEnvironment;
use crate::interpreter::*;
use crate::parser::ParameterFormals;
use std::collections::HashMap;

fn add<R: RealNumberInternalTrait, E: IEnvironment<R>>(
    arguments: impl IntoIterator<Item = Value<R, E>>,
) -> Result<Value<R, E>> {
    arguments
        .into_iter()
        .try_fold(Value::Number(Number::Integer(0)), |a, b| match (a, b) {
            (Value::Number(num1), Value::Number(num2)) => Ok(Value::Number(num1 + num2)),
            o => logic_error!("expect a number, got {}", o.1),
        })
}

fn sub<R: RealNumberInternalTrait, E: IEnvironment<R>>(
    arguments: impl IntoIterator<Item = Value<R, E>>,
) -> Result<Value<R, E>> {
    let mut iter = arguments.into_iter();
    let init = match iter.next() {
        None => logic_error!("'-' needs at least one argument"),
        Some(first) => match first {
            Value::Number(first_num) => match iter.next() {
                Some(second) => match second {
                    Value::Number(second_num) => Value::Number(first_num - second_num),
                    o => logic_error!("expect a number, got {}", o),
                },
                None => Value::Number(Number::Integer(0) - first_num),
            },
            o => logic_error!("expect a number, got {}", o),
        },
    };
    iter.try_fold(init, |a, b| match (a, b) {
        (Value::Number(num1), Value::Number(num2)) => Ok(Value::Number(num1 - num2)),
        o => logic_error!("expect a number, got {}", o.1),
    })
}

fn mul<R: RealNumberInternalTrait, E: IEnvironment<R>>(
    arguments: impl IntoIterator<Item = Value<R, E>>,
) -> Result<Value<R, E>> {
    let mut iter = arguments.into_iter();
    iter.try_fold(Value::Number(Number::Integer(1)), |a, b| match (a, b) {
        (Value::Number(num1), Value::Number(num2)) => Ok(Value::Number(num1 * num2)),
        o => logic_error!("expect a number, got {}", o.1),
    })
}

fn div<R: RealNumberInternalTrait, E: IEnvironment<R>>(
    arguments: impl IntoIterator<Item = Value<R, E>>,
) -> Result<Value<R, E>> {
    let mut iter = arguments.into_iter();
    let init = match iter.next() {
        None => logic_error!("'/' needs at least one argument"),
        Some(first) => match first {
            Value::Number(first_num) => match iter.next() {
                Some(second) => match second {
                    Value::Number(second_num) => Value::Number((first_num / second_num)?),
                    o => logic_error!("expect a number, got {}", o),
                },
                None => Value::Number((Number::Integer(1) / first_num)?),
            },
            o => logic_error!("expect a number, got {}", o),
        },
    };
    iter.try_fold(init, |a, b| match (a, b) {
        (Value::Number(num1), Value::Number(num2)) => Ok(Value::Number((num1 / num2)?)),
        o => logic_error!("expect a number, got {}", o.1),
    })
}
macro_rules! numeric_one_argument {
    ($name:tt, $func:tt$(, $err_handle:tt)?) => {
        fn $func<R: RealNumberInternalTrait, E: IEnvironment<R>>(
            arguments: impl IntoIterator<Item = Value<R, E>>,
        ) -> Result<Value<R, E>> {
            match arguments.into_iter().next() {
                Some(Value::Number(number)) => Ok(Value::Number(number.$func()$($err_handle)?)),
                Some(other) => logic_error!("{} requires a number, got {:?}", $name, other),
                _ => logic_error!("{} takes exactly one argument", $name),
            }
        }
    };
}
numeric_one_argument!("sqrt", sqrt);

numeric_one_argument!("floor", floor);

numeric_one_argument!("ceiling", ceiling);

numeric_one_argument!("exact", exact, ?);
#[test]
fn buildin_numeric_one() {
    {
        let arguments: Vec<Value<f32, StandardEnv<_>>> =
            vec![Value::Number(Number::Rational(-49, 3))];
        assert_eq!(floor(arguments), Ok(Value::Number(Number::Integer(-17))));
    }
    {
        let arguments: Vec<Value<f32, StandardEnv<_>>> = vec![Value::String("foo".to_string())];
        assert_eq!(
            sqrt(arguments),
            Err(SchemeError {
                location: None,
                category: ErrorType::Logic,
                message: "sqrt requires a number, got String(\"foo\")".to_string(),
            })
        );
    }
    {
        let arguments: Vec<Value<f32, StandardEnv<_>>> = vec![];
        assert_eq!(
            floor(arguments),
            Err(SchemeError {
                location: None,
                category: ErrorType::Logic,
                message: "floor takes exactly one argument".to_string(),
            })
        );
    }
}

macro_rules! numeric_two_arguments {
    ($name:tt, $func:tt$(, $err_handle:tt)?) => {
        fn $func<R: RealNumberInternalTrait, E: IEnvironment<R>>(
            arguments: impl IntoIterator<Item = Value<R, E>>,
        ) -> Result<Value<R, E>> {
            let mut iter = arguments.into_iter();
            let lhs = match iter.next() {
                Some(Value::Number(number)) => number,
                Some(_) => logic_error!("expect a number!"),
                _ => logic_error!("{} takes exactly two arguments", $name),
            };
            let rhs = match iter.next() {
                Some(Value::Number(number)) => number,
                Some(_) => logic_error!("expect a number!"),
                _ => logic_error!("{} takes exactly two arguments", $name),
            };
            Ok(Value::Number(lhs.$func(rhs)$($err_handle)?))
        }
    };
}

numeric_two_arguments!("floor-quotient", floor_quotient, ?);

numeric_two_arguments!("floor-remainder", floor_remainder, ?);
#[test]
fn buildin_numeric_two() {
    {
        let arguments: Vec<Value<f32, StandardEnv<_>>> = vec![
            Value::Number(Number::Integer(8)),
            Value::Number(Number::Integer(3)),
        ];
        assert_eq!(
            floor_remainder(arguments),
            Ok(Value::Number(Number::Integer(2)))
        );
    }
    {
        let arguments: Vec<Value<f32, StandardEnv<_>>> = vec![
            Value::String("foo".to_string()),
            Value::String("bar".to_string()),
        ];
        assert_eq!(
            floor_quotient(arguments),
            Err(SchemeError {
                location: None,
                category: ErrorType::Logic,
                message: "expect a number!".to_string(),
            })
        );
    }
    {
        let arguments: Vec<Value<f32, StandardEnv<_>>> = vec![Value::Number(Number::Integer(1))];
        assert_eq!(
            floor_remainder(arguments),
            Err(SchemeError {
                location: None,
                category: ErrorType::Logic,
                message: "floor-remainder takes exactly two arguments".to_string(),
            })
        );
    }
    {
        let arguments: Vec<Value<f32, StandardEnv<_>>> = vec![];
        assert_eq!(
            floor_quotient(arguments),
            Err(SchemeError {
                location: None,
                category: ErrorType::Logic,
                message: "floor-quotient takes exactly two arguments".to_string(),
            })
        );
    }
}
fn vector<R: RealNumberInternalTrait, E: IEnvironment<R>>(
    arguments: impl IntoIterator<Item = Value<R, E>>,
) -> Result<Value<R, E>> {
    let vector: Vec<Value<R, E>> = arguments.into_iter().collect();
    Ok(Value::Vector(vector))
}

fn vector_ref<R: RealNumberInternalTrait, E: IEnvironment<R>>(
    arguments: impl IntoIterator<Item = Value<R, E>>,
) -> Result<Value<R, E>> {
    let mut iter = arguments.into_iter();
    match iter.next() {
        None => logic_error!("vector_ref requires exactly two argument"),
        Some(Value::Vector(vector)) => match iter.next() {
            None => logic_error!("vector_ref requires exactly two argument"),
            Some(Value::Number(Number::Integer(i))) => match vector.get(i as usize) {
                Some(value) => Ok(value.clone()),
                None => logic_error!("vector index out of bound"),
            },
            _ => logic_error!("expect a integer!"),
        },
        _ => logic_error!("expect a vector!"),
    }
}

#[test]
fn buildin_vector_ref() {
    let vector: Value<f32, StandardEnv<_>> = Value::Vector(vec![
        Value::Number(Number::Integer(5)),
        Value::String("foo".to_string()),
        Value::Number(Number::Rational(5, 3)),
    ]);
    {
        let arguments = vec![vector.clone(), Value::Number(Number::Integer(0))];
        assert_eq!(vector_ref(arguments), Ok(Value::Number(Number::Integer(5))));
    }
    {
        let arguments = vec![vector.clone(), Value::Number(Number::Integer(1))];
        assert_eq!(vector_ref(arguments), Ok(Value::String("foo".to_string())));
    }
    {
        let arguments = vec![vector.clone(), Value::Number(Number::Integer(2))];
        assert_eq!(
            vector_ref(arguments),
            Ok(Value::Number(Number::Rational(5, 3)))
        );
    }
    {
        let arguments = vec![vector.clone(), Value::Number(Number::Integer(3))];
        assert_eq!(
            vector_ref(arguments),
            Err(SchemeError {
                location: None,
                category: ErrorType::Logic,
                message: "vector index out of bound".to_string(),
            })
        );
    }
    {
        let arguments = vec![vector.clone(), Value::Number(Number::Real(1.5))];
        assert_eq!(
            vector_ref(arguments),
            Err(SchemeError {
                location: None,
                category: ErrorType::Logic,
                message: "expect a integer!".to_string(),
            })
        );
    }
    {
        let arguments: Vec<Value<f32, StandardEnv<_>>> = vec![
            Value::Number(Number::Integer(1)),
            Value::Number(Number::Integer(1)),
        ];
        assert_eq!(
            vector_ref(arguments),
            Err(SchemeError {
                location: None,
                category: ErrorType::Logic,
                message: "expect a vector!".to_string(),
            })
        );
    }
    {
        let arguments: Vec<Value<f32, StandardEnv<_>>> = vec![];
        assert_eq!(
            vector_ref(arguments),
            Err(SchemeError {
                location: None,
                category: ErrorType::Logic,
                message: "vector_ref requires exactly two argument".to_string(),
            })
        );
    }
    {
        let arguments: Vec<Value<f32, StandardEnv<_>>> = vec![vector];
        assert_eq!(
            vector_ref(arguments),
            Err(SchemeError {
                location: None,
                category: ErrorType::Logic,
                message: "vector_ref requires exactly two argument".to_string(),
            })
        );
    }
}

fn display<R: RealNumberInternalTrait, E: IEnvironment<R>>(
    arguments: impl IntoIterator<Item = Value<R, E>>,
) -> Result<Value<R, E>> {
    Ok(match arguments.into_iter().next() {
        Some(value) => {
            print!("{}", value);
            Value::Void
        }
        None => logic_error!("display takes exactly one argument"),
    })
}

fn newline<R: RealNumberInternalTrait, E: IEnvironment<R>>(
    arguments: impl IntoIterator<Item = Value<R, E>>,
) -> Result<Value<R, E>> {
    Ok(match arguments.into_iter().next() {
        None => {
            println!("");
            Value::<R, E>::Void
        }
        _ => logic_error!("display takes exactly one argument"),
    })
}

macro_rules! comparision {
    ($name:tt, $operator:tt) => {
        fn $name<R: RealNumberInternalTrait, E: IEnvironment<R>>(
                        arguments: impl IntoIterator<Item = Value<R, E>>
        ) -> Result<Value<R, E>> {
            let mut iter = arguments.into_iter();
            match iter.next() {
                None => Ok(Value::Boolean(true)),
                Some(first) => {
                            let mut last = first;
                            for current in iter {
                                match (last, current) {
                                    (Value::Number(a), Value::Number(b)) => {
                                        if !(a $operator b) {
                                            return Ok(Value::Boolean(false));
                                        }
                                        last = Value::Number(b);
                                    }
                                    _ => logic_error!("{} comparision can only between numbers!", stringify!($operator)),
                                }
                            }
                            Ok(Value::Boolean(true))
                        }

            }
        }
    }
}

comparision!(equals, ==);
comparision!(greater, >);
comparision!(greater_equal, >=);
comparision!(less, <);
comparision!(less_equal, <=);

macro_rules! first_of_order {
    ($name:tt, $cmp:tt) => {
        fn $name<R: RealNumberInternalTrait, E: IEnvironment<R>>(
                        arguments: impl IntoIterator<Item = Value<R, E>>
        ) -> Result<Value<R, E>> {
            let mut iter = arguments.into_iter();
            match iter.next() {
                None => logic_error!("min requires at least one argument!"),
                Some(Value::Number(num)) => {
                    iter.try_fold(Value::Number(num), |a, b| match (a, b) {
                                (Value::Number(num1), Value::Number(num2)) => {
                                    Ok(Value::Number(match num1 $cmp num2 {
                                        true => upcast_oprands((num1, num2)).lhs(),
                                        false => upcast_oprands((num1, num2)).rhs(),
                                    }))
                                }
                                o => logic_error!("expect a number, got {}", o.1),
                            })
                        },
                Some(o) => logic_error!("expect a number, got {}", o),
                }
            }
        }
}

first_of_order!(max, >);
first_of_order!(min, <);

pub fn base_library<'a, R: RealNumberInternalTrait, E: IEnvironment<R>>(
) -> HashMap<String, Value<R, E>> {
    macro_rules! function_mapping {
        ($ident:tt, $fixed_parameter:expr, $variadic_parameter:expr, $function:tt) => {
            (
                $ident.to_owned(),
                Value::Procedure(Procedure::new_buildin_pure(
                    $ident,
                    ParameterFormals($fixed_parameter, $variadic_parameter),
                    $function,
                )),
            )
        };
    }

    vec![
        function_mapping!("+", vec![], Some("x".to_string()), add),
        function_mapping!("-", vec![], Some("x".to_string()), sub),
        function_mapping!("*", vec![], Some("x".to_string()), mul),
        function_mapping!("/", vec![], Some("x".to_string()), div),
        function_mapping!("=", vec![], Some("x".to_string()), equals),
        function_mapping!("<", vec![], Some("x".to_string()), less),
        function_mapping!("<=", vec![], Some("x".to_string()), less_equal),
        function_mapping!(">", vec![], Some("x".to_string()), greater),
        function_mapping!(">=", vec![], Some("x".to_string()), greater_equal),
        function_mapping!("min", vec![], Some("x".to_string()), min),
        function_mapping!("max", vec![], Some("x".to_string()), max),
        function_mapping!("sqrt", vec!["x".to_string()], None, sqrt),
        function_mapping!("floor", vec!["x".to_string()], None, floor),
        function_mapping!("ceiling", vec!["x".to_string()], None, ceiling),
        function_mapping!("exact", vec!["x".to_string()], None, exact),
        function_mapping!(
            "floor-quotient",
            vec!["n1".to_string(), "n2".to_string()],
            None,
            floor_quotient
        ),
        function_mapping!(
            "floor-remainder",
            vec!["n1".to_string(), "n2".to_string()],
            None,
            floor_remainder
        ),
        function_mapping!("display", vec!["value".to_string()], None, display),
        function_mapping!("newline", vec![], None, newline),
        function_mapping!("vector", vec![], None, vector),
        function_mapping!(
            "vector-ref",
            vec!["vector".to_string(), "k".to_string()],
            None,
            vector_ref
        ),
    ]
    .into_iter()
    .collect()
}

#[test]
fn buildin_parameters_length() -> Result<()> {
    let buildin_functions = base_library::<f32, StandardEnv<_>>();
    assert!(matches!(
        &buildin_functions["sqrt"],
        Value::Procedure(Procedure::Buildin(sqrt)) if sqrt.parameters.0.len() == 1));
    assert!(matches!(
        &buildin_functions["display"],
        Value::Procedure(Procedure::Buildin(display)) if display.parameters.0.len() == 1));
    assert!(matches!(
        &buildin_functions["newline"],
        Value::Procedure(Procedure::Buildin(newline)) if newline.parameters.0.len() == 0));
    Ok(())
}
