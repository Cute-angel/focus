use std::any::Any;
use std::collections::HashMap;
use std::string::ParseError;
use crate::api::command_tree::{CommandDispatcher, CommandNode, StringArgument};
use crate::api::extension::{action, Extension, ExtensionResult};
use crate::extension::cal_plugin::CalculatorError::{DivisionByZeroError, FormatError, LessOperatorError, OperatorLocationError, ParenCloseError, Unknown};

#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Operator(char),
    LParen,
    RParen,
}

#[derive(Debug,thiserror::Error)]
pub enum CalculatorError{
    #[error("Number parse error")]
    FormatError,
    #[error("Unknown character {0}")]
    Unknown(char),
    #[error("Parse not closed")]
    ParenCloseError,
    #[error("less operator")]
    LessOperatorError,
    #[error("operator location error")]
    OperatorLocationError,
    #[error("{0} cannot be divided by zero")]
    DivisionByZeroError(f64)

}

fn tokenize(expr: &str) -> Result<Vec<Token>, CalculatorError> {
    let expr = expr.trim_matches('=');
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();
    let mut num_buf = String::new();

    while let Some(&c) = chars.peek() {
        match c {
            '0'..='9' | '.' => {
                num_buf.push(c);
                chars.next();
            }
            '+' | '-' => {
                // 区分负号与减号
                if num_buf.is_empty() {
                    // 检查是否是负号（前面没有数字或右括号）
                    let prev_is_num = tokens.last().map(|t| matches!(t, Token::Number(_)|Token::RParen)).unwrap_or(false);
                    if !prev_is_num {
                        num_buf.push(c);
                        chars.next();
                        continue;
                    }
                }
                if !num_buf.is_empty() {
                    tokens.push(Token::Number(num_buf.parse::<f64>().map_err(|_| FormatError)?));
                    num_buf.clear();
                }
                tokens.push(Token::Operator(c));
                chars.next();
            }
            '*' | '/' | '^' => {
                if !num_buf.is_empty() {
                    tokens.push(Token::Number(num_buf.parse::<f64>().map_err(|_| FormatError)?));
                    num_buf.clear();
                }
                tokens.push(Token::Operator(c));
                chars.next();
            }
            '(' => {
                if !num_buf.is_empty() {
                    tokens.push(Token::Number(num_buf.parse::<f64>().map_err(|_| FormatError)?));
                    num_buf.clear();
                }
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                if !num_buf.is_empty() {
                    tokens.push(Token::Number(num_buf.parse::<f64>().map_err(|_| FormatError)?));
                    num_buf.clear();
                }
                tokens.push(Token::RParen);
                chars.next();
            }
            ' ' => {
                if !num_buf.is_empty() {
                    tokens.push(Token::Number(num_buf.parse::<f64>().map_err(|_| FormatError)?));
                    num_buf.clear();
                }
                chars.next();
            }
            _ => return Err(Unknown(c)),
        }
    }

    if !num_buf.is_empty() {
        tokens.push(Token::Number(num_buf.parse::<f64>().map_err(|_| FormatError)?));
    }

    Ok(tokens)
}

fn check_syntax(tokens: &[Token]) -> Result<(), CalculatorError> {
    let mut paren_balance = 0;
    let mut prev_was_op = true;

    for token in tokens {
        match token {
            Token::LParen => {
                paren_balance += 1;
                prev_was_op = true;
            }
            Token::RParen => {
                paren_balance -= 1;
                if paren_balance < 0 {
                    return Err(ParenCloseError);
                }
                prev_was_op = false;
            }
            Token::Operator(_) => {
                if prev_was_op {
                    return Err( LessOperatorError);
                }
                prev_was_op = true;
            }
            Token::Number(_) => {
                prev_was_op = false;
            }
        }
    }

    if paren_balance != 0 {
        return Err(ParenCloseError);
    }
    if prev_was_op {
        return Err(OperatorLocationError);
    }

    Ok(())
}

fn precedence(op: char) -> i32 {
    match op {
        '^' => 3,
        '*' | '/' => 2,
        '+' | '-' => 1,
        _ => 0,
    }
}

fn to_rpn(tokens: &[Token]) -> Vec<Token> {
    let mut output = Vec::new();
    let mut ops = Vec::new();

    for token in tokens {
        match token {
            Token::Number(_) => output.push(token.clone()),
            Token::Operator(op) => {
                while let Some(Token::Operator(top)) = ops.last() {
                    if precedence(*top) >= precedence(*op) && *op != '^' {
                        output.push(ops.pop().unwrap());
                    } else {
                        break;
                    }
                }
                ops.push(token.clone());
            }
            Token::LParen => ops.push(token.clone()),
            Token::RParen => {
                while let Some(t) = ops.pop() {
                    if let Token::LParen = t {
                        break;
                    } else {
                        output.push(t);
                    }
                }
            }
        }
    }

    while let Some(t) = ops.pop() {
        output.push(t);
    }

    output
}

fn eval_rpn(rpn: &[Token]) -> Result<f64, CalculatorError> {
    let mut stack = Vec::new();

    for token in rpn {
        match token {
            Token::Number(n) => stack.push(*n),
            Token::Operator(op) => {
                if stack.len() < 2 {
                    return Err(LessOperatorError);
                }
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let res = match op {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    '/' => {
                        if b == 0.0 {
                            return Err(DivisionByZeroError(a));
                        }
                        a / b
                    }
                    '^' => a.powf(b),
                    _ => return Err(Unknown(op.clone())),
                };
                stack.push(res);
            }
            _ => {}
        }
    }

    if stack.len() != 1 {
        return Err(FormatError);
    }

    Ok(stack[0])
}

pub fn evaluate_expression(expr: &str) -> Result<f64, CalculatorError> {

    let tokens = tokenize(expr)?;
    check_syntax(&tokens)?;
    let rpn = to_rpn(&tokens);
    eval_rpn(&rpn)
}
#[derive(Default)]
pub(crate) struct Calculator;



impl Extension for Calculator {
    fn OnMount<>(&self, command_dispatcher: &mut CommandDispatcher) {
        let chipboard_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15.666 3.888A2.25 2.25 0 0 0 13.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 0 1-.75.75H9a.75.75 0 0 1-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 0 1-2.25 2.25H6.75A2.25 2.25 0 0 1 4.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 0 1 1.927-.184" />
            </svg>
            "#;

        let cmd = CommandNode::new("cal")
            .then(
                CommandNode::new("cal_expression")
                    .set_truncate()
                    .argument(StringArgument)
                    .execute(|ctx|{
                        if let Some(exp) = ctx.get_parm("cal_expression"){
                            match evaluate_expression(exp) {
                                Ok(val)=>{
                                    println!("{}", val);
                                    let res =  ExtensionResult{
                                        icon:"a".to_string(),
                                        title:val.to_string(),
                                        description:"Press Enter to copy to clipboard".to_string(),
                                        actions:vec![action {
                                            icon:chipboard_svg.to_string(),
                                            tooltip:"Enter".to_string(),
                                            value:val.to_string(),
                                        }],
                                    };
                                    Box::new(res) as Box<dyn Any>
                                },
                                Err(e)=>{
                                    Box::new(e) as Box<dyn Any>
                                }
                            }
                        }else{
                            Box::new("error param can't get") as Box<dyn Any>
                        }

                    })
            );

        command_dispatcher.register(cmd);
    }

    fn OnUnmount<>(&self, command_dispatcher: &mut CommandDispatcher) {}
}


#[cfg(test)]
mod tests{
    use crate::extension::cal_plugin::evaluate_expression;

    #[test]
    fn test_evaluate_expression() {
        let exprs = [
            "3 + 4 * 2 / (1 - 5)^2",
            "-3.5 * (2 + 4)^2",
            "((2+3)*4)^2",
            "3 + * ", // 错误示例
            "1 + .2"
        ];

        for expr in exprs {
            match evaluate_expression(expr) {
                Ok(v) => println!("{} = {}", expr, v),
                Err(e) => println!("{} ❌ {}", expr, e),
            }
        }
    }

}

