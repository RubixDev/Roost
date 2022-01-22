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
