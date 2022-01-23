use super::value::Value;

pub fn print(args: Vec<Value>) -> Value {
    let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    print!("{}", args.join(" "));
    return Value::Void;
}

pub fn printl(args: Vec<Value>) -> Value {
    let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    println!("{}", args.join(" "));
    return Value::Void;
}

pub fn type_of(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        panic!(
            "TypeError at position {{}}: Function 'typeOf' takes 1 argument, however {} were supplied",
            args.len(),
        );
    }

    return Value::String(super::value::types::type_of(&args[0]).to_string());
}
