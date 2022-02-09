use std::io::Cursor;
use ntest::timeout;

use crate::{interpreter::Interpreter, parser::Parser, lexer::Lexer};

fn test_code(code: &str, expected: &str) {
    let mut out = Cursor::new(vec![]);

    match Interpreter::new_run(
        match Parser::new_parse(Lexer::new(&code.to_owned(), String::from("test"))) {
            Ok(node) => node,
            Err(e) => panic!("{:?}", e),
        },
        &mut out,
        |_| {},
    ) {
        Ok(_) => {},
        Err(e) => panic!("{}", e),
    };

    assert_eq!(std::str::from_utf8(out.get_ref()).unwrap(), expected);
}

#[test]
fn assignments() {
    test_code(r#"
    var start = 12
    print(start + ' ')

    start  +=  2; print(start + ' ') // 14
    start  -=  4; print(start + ' ') // 10
    start  *=  2; print(start + ' ') // 20
    start  /=  2; print(start + ' ') // 10
    start  %=  3; print(start + ' ') // 1
    start   = 10; print(start + ' ') // 10
    start  \=  3; print(start + ' ') // 3
    start **=  3; print(start + ' ') // 27
    "#, "12 14 10 20 10 1 10 3 27 ")
}

#[test]
fn operators() {
    test_code(r#"
    print(10 + 3, '')
    print(10 - 3, '')
    print(10 * 3, '')
    print(10 / 3, '')
    print(10 % 3, '')
    print(10 \ 3, '')
    print(10 ** 3, '')

    print(+5, '')
    print(-5, '')
    print(!5, '')

    print(5 < 5, '')
    print(5 <= 5, '')
    print(5 > 5, '')
    print(5 >= 5, '')
    print(5 == 5, '')
    print(5 != 5, '')
    print(false | true, '')
    print(false & true, '')
    "#, "13 7 30 3.3333333333333333333333333333 1 3 1000 5 -5 false false true false true true false true false ")
}

#[test]
#[timeout(20_000)]
fn loops() {
    test_code(r#"
    var i = 0
    loop { if (i > 50) break; i += 1 }
    var i = 0
    while (i <= 50) { i += 1 }
    var i = 0
    while (i <= 50) i += 1
    for (i in 0..=50) { continue; 10/0 }
    "#, "")
}

#[test]
fn fun() {
    test_code(r#"
    fun a(a, b) return a + b
    print(a(3, 4), '')
    fun a(a, b) { return a + b; 10/0 }
    print(a(3, 4), '')
    fun a(a, b) a + b
    print(a(3, 4), '')
    fun a(a, b) { a + b }
    print(a(3, 4), '')

    var a = fun(a, b) return a + b
    print(a(3, 4), '')
    var a = fun(a, b) { return a + b; 10/0 }
    print(a(3, 4), '')
    "#, "7 7 7 7 7 7 ")
}
