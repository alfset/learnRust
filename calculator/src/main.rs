use std::io;

enum Operation {
    Add(f64, f64),
    Subtract(f64, f64),
    Multiply(f64, f64),
    Divide(f64, f64),
}

fn calculate(op: Operation) -> f64 {
    match op {
        Operation::Add(a, b) => a + b,
        Operation::Subtract(a, b) => a - b,
        Operation::Multiply(a, b) => a * b,
        Operation::Divide(a, b) => {
            if b == 0.0 {
                println!("Error: Division by zero!");
                0.0
            } else {
                a / b
            }
        }
    }
}

fn main() {
    println!("Enter the first number:");
    let mut input1 = String::new();
    io::stdin().read_line(&mut input1).expect("Failed to read line");
    let num1: f64 = input1.trim().parse().expect("Please enter a valid number");

    println!("Enter the operation (+, -, *, /):");
    let mut op_input = String::new();
    io::stdin().read_line(&mut op_input).expect("Failed to read line");
    let op = op_input.trim();

    println!("Enter the second number:");
    let mut input2 = String::new();
    io::stdin().read_line(&mut input2).expect("Failed to read line");
    let num2: f64 = input2.trim().parse().expect("Please enter a valid number");

    let operation = match op {
        "+" => Operation::Add(num1, num2),
        "-" => Operation::Subtract(num1, num2),
        "*" => Operation::Multiply(num1, num2),
        "/" => Operation::Divide(num1, num2),
        _ => {
            println!("Invalid operation entered.");
            return;
        }
    };
    let result = calculate(operation);
    println!("Result: {}", result);
}
