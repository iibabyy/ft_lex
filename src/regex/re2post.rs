use super::*;

/// Convert infix regex to postfix
pub fn re2post(mut tokens: VecDeque<TokenType>) -> ParsingResult<Vec<TokenType>> {
    let mut operator_stack: Vec<TokenType> = Vec::with_capacity(tokens.len());
    let mut output_stack: Vec<TokenType> = Vec::with_capacity(tokens.len());

    while let Some(token) = tokens.pop_front() {
        match token {
            TokenType::Literal(type_) => output_stack.push(TokenType::Literal(type_)),

            TokenType::OpenParenthesis(_) => {
                operator_stack.push(token);
            }

            TokenType::CloseParenthesis(_) => {
                loop {
                    let next_operator = operator_stack.last();

                    match next_operator {
                        // Open parenthesis found
                        Some(&TokenType::OpenParenthesis(_)) => break,

                        // Push all operator if not parenthesis
                        Some(_) => output_stack.push(operator_stack.pop().unwrap()),

                        // Open parenthesis not found
                        None => return Err(ParsingError::unrecognized_rule().because("Unclosed parenthesis")),
                    }
                }

                // Remove Open parenthesis if found
                operator_stack.pop();
            }

            // Other operator
            token => {
				// Compare precedence of operators (shunting-yard algorithm)
                while let Some(next_operator) = operator_stack.last() {
                    if next_operator.precedence() >= token.precedence() {
                        output_stack.push(operator_stack.pop().unwrap());
                    } else {
                        break;
                    }
                }

                operator_stack.push(token);
            }
        }
    }

    // Check for unclosed parentheses
    while let Some(token) = operator_stack.pop() {
        if matches!(token, TokenType::OpenParenthesis(_)) {
            return Err(ParsingError::unrecognized_rule().because("Unclosed parenthesis"));
        }

        output_stack.push(token);
    }

    return Ok(output_stack);
}
