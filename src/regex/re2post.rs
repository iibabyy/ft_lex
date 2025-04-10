use super::*;

/// Convert infix regex to postfix
pub fn re2post(mut tokens: VecDeque<TokenType>) -> ParsingResult<VecDeque<TokenType>> {
    let mut operator_stack: Vec<TokenType> = Vec::with_capacity(tokens.len());
    let mut output_stack: VecDeque<TokenType> = VecDeque::with_capacity(tokens.len());

	let mut line_start_found = false;
	let mut line_end_found = false;

    while let Some(token) = tokens.pop_front() {
        match token {
            TokenType::Literal(type_) => output_stack.push_back(TokenType::Literal(type_)),

			TokenType::StartOrEndCondition(RegexType::LineStart) => {
				// '^' special character must be the first character of the regex
				if output_stack.back().is_some() || operator_stack.last().is_some() {
					return Err(ParsingError::unrecognized_rule().because("Unexpected '^' special character"));
				}

				// Duplicate '^' special character (only one is allowed)
				if line_start_found {
					return Err(ParsingError::unrecognized_rule().because("Unexpected '^' special character"));
				}

				line_start_found = true;
			}

			TokenType::StartOrEndCondition(RegexType::LineEnd) => {
				if tokens.front().is_some() {
					return Err(ParsingError::unrecognized_rule().because("Unexpected '$' special character"));
				}

				// Duplicate '$' special character (only one is allowed)
				if line_end_found {
					return Err(ParsingError::unrecognized_rule().because("Unexpected '$' special character"));
				}

				line_end_found = true;
			}

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
                        Some(_) => output_stack.push_back(operator_stack.pop().unwrap()),

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
                        output_stack.push_back(operator_stack.pop().unwrap());
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

        output_stack.push_back(token);
    }

	if line_start_found {
		output_stack.push_front(TokenType::StartOrEndCondition(RegexType::LineStart));
	}

	if line_end_found {
		output_stack.push_back(TokenType::StartOrEndCondition(RegexType::LineEnd));
	}

    return Ok(output_stack);
}
