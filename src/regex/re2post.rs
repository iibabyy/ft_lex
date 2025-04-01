use super::*;

pub fn re2post(mut tokens: VecDeque<TokenType>) -> ParsingResult<VecDeque<TokenType>> {
    let mut operator_stack: Vec<TokenType> = Vec::with_capacity(tokens.len());
    let mut output_stack: Vec<TokenType> = Vec::with_capacity(tokens.len());



    while let Some(token) = tokens.pop_front() {
        match token {
            TokenType::OpenParenthesis(_) => {
                operator_stack.push(token);
            },

            TokenType::CloseParenthesis(_) => {
                loop {
                    let next_operator = operator_stack.last();

                    match next_operator {

                        Some(&TokenType::OpenParenthesis(_)) => break, 

                        Some(_) => output_stack.push(operator_stack.pop().unwrap()),

                        None => return Err(ParsingError::unrecognized_rule().because(""))
                    }
                }
            },

            _ => todo!()
        }
    }

    todo!()
}