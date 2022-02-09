use std::io::Cursor;

use crate::{interpreter::Interpreter, parser::Parser, lexer::Lexer};

#[test]
fn assignments() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
    var start = 12
    printl(start)

    start  +=  2; printl(start) // 14
    start  -=  4; printl(start) // 10
    start  *=  2; printl(start) // 20
    start  /=  2; printl(start) // 10
    start  %=  3; printl(start) // 1
    start   = 10; printl(start) // 10
    start  \=  3; printl(start) // 3
    start **=  3; printl(start) // 27
    "#.to_owned();

    let mut out = Cursor::new(vec![]);

    match Interpreter::new_run(
        match Parser::new_parse(Lexer::new(&code, String::from("test"))) {
            Ok(node) => node,
            Err(e) => panic!("{:?}", e),
        },
        &mut out,
        |_| {},
    ) {
        Ok(_) => {},
        Err(e) => panic!("{}", e),
    };

    assert_eq!(std::str::from_utf8(out.get_ref()).unwrap(), "12
14
10
20
10
1
10
3
27
");

    Ok(())
}
