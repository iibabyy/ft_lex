use crate::regex::*;

#[cfg(test)]
mod add_state_with_memo_iterative_tests {
    use super::*;
    use std::{collections::HashSet, rc::Rc};

    // Helper function to create a deep linear chain of states
    fn create_deep_chain(depth: usize) -> StatePtr {
        let match_state = State::match_();
        if depth == 0 {
            return match_state;
        }
        
        let mut current = match_state;
        for i in 0..depth {
            let char_val = (b'a' + (i % 26) as u8) as char;
            let basic = State::basic(RegexType::Char(char_val));
            basic.borrow_mut().into_basic().unwrap().out.replace(current);
            current = basic;
        }
        
        current
    }
    
    // Helper to create a deeply nested split tree
    fn create_split_tree(depth: usize) -> StatePtr {
        if depth == 0 {
            return State::match_();
        }
        
        let left = State::basic(RegexType::Char('a'));
        let right = create_split_tree(depth - 1);
        
        State::split(left, right)
    }
    
    // Helper to count states in a StateList
    fn count_states(list: &StateList) -> usize {
        list.iter().count()
    }
    
    // Helper to check if StateList contains specific character states
    fn contains_char(list: &StateList, c: char) -> bool {
        list.iter().any(|state| {
            if let Some(basic) = state.borrow().into_basic() {
                if let Some(ch) = basic.c.char() {
                    return ch == c;
                }
            }
            false
        })
    }

    // Helper to add all states in a chain to a list
    fn add_all_states_in_chain(list: &mut StateList, state: &StatePtr, visited: &mut HashSet<*const State>) {
        let mut current = Rc::clone(state);
        
        while !State::is_none_ptr(&current) && !State::is_match_ptr(&current) {
            list.add_state_with_memo_iterative(&current, visited);
            
			let tmp = current;
			let borrow = tmp.borrow();
            if let Some(basic) = &borrow.into_basic() {
                current = Rc::clone(&basic.out.borrow());
            } else {
				drop(borrow);
				current = tmp;
                break;
            }
        }
        
        if State::is_match_ptr(&current) {
            list.add_state_with_memo_iterative(&current, visited);
        }
    }

    #[test]
    fn test_basic_state_add() {
        // Create a basic state
        let basic = State::basic(RegexType::Char('x'));
        
        // Add to list using iterative method
        let mut list = StateList::new();
        list.add_state_with_memo_iterative(&basic, &mut HashSet::new());
        
        // Verify list has exactly one state
        assert_eq!(count_states(&list), 1);
        assert!(contains_char(&list, 'x'));
    }
    
    #[test]
    fn test_match_state_add() {
        // Create a match state
        let match_state = State::match_();
        
        // Add to list using iterative method
        let mut list = StateList::new();
        list.add_state_with_memo_iterative(&match_state, &mut HashSet::new());
        
        // Verify list has exactly one state
        assert_eq!(count_states(&list), 1);
        assert!(list.is_matched());
    }

    #[test]
    fn test_simple_split_state() {
        // Create a split with two basic states
        let a = State::basic(RegexType::Char('a'));
        let b = State::basic(RegexType::Char('b'));
        let split = State::split(a, b);
        
        // Add to list using iterative method
        let mut list = StateList::new();
        let mut visited = HashSet::new();
        
        // Add split state - this should add out1 and out2 states
        list.add_state_with_memo_iterative(&split, &mut visited);
        
        // Verify list has exactly two states (a, b) but not the split itself
        assert_eq!(count_states(&list), 2);
        assert!(contains_char(&list, 'a'));
        assert!(contains_char(&list, 'b'));
    }
    
    #[test]
    fn test_nested_split_state() {
        // Create a nested split: split(a, split(b, c))
        let a = State::basic(RegexType::Char('a'));
        let b = State::basic(RegexType::Char('b'));
        let c = State::basic(RegexType::Char('c'));
        let inner_split = State::split(b, c);
        let outer_split = State::split(a, inner_split);
        
        // Add to list using iterative method
        let mut list = StateList::new();
        let mut visited = HashSet::new();
        
        // Add outer split - this should add 'a' and process inner_split
        list.add_state_with_memo_iterative(&outer_split, &mut visited);
        
        // Verify list has exactly three states (a, b, c) but not the splits
        assert_eq!(count_states(&list), 3);
        assert!(contains_char(&list, 'a'));
        assert!(contains_char(&list, 'b'));
        assert!(contains_char(&list, 'c'));
    }
    
    #[test]
    fn test_already_visited_state() {
        // Create a state that would be visited multiple times
        let shared = State::basic(RegexType::Char('x'));
        
        // Create a structure that visits the shared state twice
        let a = State::basic(RegexType::Char('a'));
        a.borrow_mut().into_basic().unwrap().out.replace(shared.clone());
        
        let b = State::basic(RegexType::Char('b'));
        b.borrow_mut().into_basic().unwrap().out.replace(shared.clone());
        
        let split = State::split(Rc::clone(&a), b);
        
        // First try without the visited set to confirm duplicate detection
        let mut list1 = StateList::new();
        let mut visited = HashSet::new();
        list1.add_state_with_memo_iterative(&split, &mut visited);

        // Verify shared state is only added once - we should have 2 states: a, b
        assert_eq!(count_states(&list1), 2);
        assert!(contains_char(&list1, 'a'));
        assert!(contains_char(&list1, 'b'));

        // Adding 'a' and 'b' states out (x). This should add 'x' state one time
		let borrow = a.borrow();
        list1.add_state_with_memo_iterative(&borrow.into_basic().unwrap().out.borrow(), &mut visited);
        
        // Verify 'x' is now added
        assert_eq!(count_states(&list1), 3);
        assert!(contains_char(&list1, 'a'));
        assert!(contains_char(&list1, 'b'));
        assert!(contains_char(&list1, 'x'));
    }
    
    #[test]
    fn test_contains_check() {
        // Create a state
        let a = State::basic(RegexType::Char('a'));
        
        // Add it to the list first
        let mut list = StateList::new();
        list.push(&a);

        // Now try to add it again with the iterative method
        list.add_state_with_memo_iterative(&a, &mut HashSet::new());
        
        // Verify it's still only in the list once
        assert_eq!(count_states(&list), 1);
    }
    
    #[test]
    fn test_deep_structure() {
        // Create a deep structure that would likely cause stack overflow with recursion
        let deep_chain = create_deep_chain(1000);

        // Add using iterative method - but we need to manually traverse the chain
        let mut list = StateList::new();
        let mut visited = HashSet::new();
        
        // Add each state in the chain one by one
        add_all_states_in_chain(&mut list, &deep_chain, &mut visited);
        
        // Verify correct number of states (should have the whole chain)
        assert_eq!(count_states(&list), 1001); // 1000 basic states + 1 match state
    }
    
    #[test]
    fn test_deep_split_tree() {
        // Create a deeply nested split tree
        let split_tree = create_split_tree(100);
        
        // Add using iterative method
        let mut list = StateList::new();
        let mut visited = HashSet::new();
        
        // Need to traverse the split tree manually
        let mut work_stack = vec![Rc::clone(&split_tree)];
        
        while let Some(current) = work_stack.pop() {
            list.add_state_with_memo_iterative(&current, &mut visited);
            
			let tmp = current;
            if State::is_split_ptr(&tmp) {
				let borrow = tmp.borrow();
                let split = borrow.into_split().unwrap();
                work_stack.push(Rc::clone(&split.out1.borrow()));
                work_stack.push(Rc::clone(&split.out2.borrow()));
            } else if let Some(basic) = tmp.borrow().into_basic() {
                if !State::is_none_ptr(&basic.out.borrow()) {
                    work_stack.push(Rc::clone(&basic.out.borrow()));
                }
            }
        }
        
        // Should have 100 basic states ('a') + 1 match state
        assert_eq!(count_states(&list), 101);
    }
    
    #[test]
    fn test_compare_with_recursive() {
        // Test with structures small enough not to overflow the recursive version
        let test_cases = vec![
            State::basic(RegexType::Char('a')),
            State::match_(),
            State::split(
                State::basic(RegexType::Char('a')), 
                State::basic(RegexType::Char('b'))
            ),
            create_deep_chain(10),
            create_split_tree(5)
        ];
        
        for test_case in test_cases {
            // Create two identical lists
            let mut recursive_list = StateList::new();
            let mut iterative_list = StateList::new();
            let mut visited_recursive = HashSet::new();
            let mut visited_iterative = HashSet::new();
            
            // Fill recursive list using recursive method
            
            // For iterative list, we need to manually traverse the structure
            let mut work_stack = vec![Rc::clone(&test_case)];
            
            while let Some(current) = work_stack.pop() {
				recursive_list.add_state_with_memo(&current, &mut visited_recursive);
                iterative_list.add_state_with_memo_iterative(&current, &mut visited_iterative);
                
                if State::is_split_ptr(&current) {
					let tmp = current;
					let borrow = tmp.borrow();
                    let split = borrow.into_split().unwrap();
                    work_stack.push(Rc::clone(&split.out1.borrow()));
                    work_stack.push(Rc::clone(&split.out2.borrow()));
                } else if let Some(basic) = current.borrow().into_basic() {
                    if !State::is_none_ptr(&basic.out.borrow()) {
                        work_stack.push(Rc::clone(&basic.out.borrow()));
                    }
                }
            }
            
            // Compare results
            assert_eq!(
                count_states(&recursive_list), 
                count_states(&iterative_list),
                "State count should be identical"
            );
            
            // Verify same character states
            for state in recursive_list.iter() {
                if let Some(basic) = state.borrow().into_basic() {
                    if let Some(ch) = basic.c.char() {
                        assert!(
                            contains_char(&iterative_list, ch),
                            "Iterative list should contain char '{}'", ch
                        );
                    }
                }
            }
            
            // Check match states
            assert_eq!(
                recursive_list.is_matched(),
                iterative_list.is_matched(),
                "Match state presence should be identical"
            );
        }
    }
    
    #[test]
    fn test_processing_order() {
        // Create a split with character states a,b,c in a specific order
        let a = State::basic(RegexType::Char('a'));
        let b = State::basic(RegexType::Char('b'));
        let c = State::basic(RegexType::Char('c'));
        
        let split1 = State::split(a, b);
        let split2 = State::split(Rc::clone(&split1), c);
        
        // Add to list
        let mut list = StateList::new();
        let mut visited = HashSet::new();
        
        // Need to manually process all splits
        list.add_state_with_memo_iterative(&split2, &mut visited);
        
        // Process split1 from split2's out1
        list.add_state_with_memo_iterative(&split1, &mut visited);
        
        // Verify all three basic states are present
        assert_eq!(count_states(&list), 3);
        assert!(contains_char(&list, 'a'));
        assert!(contains_char(&list, 'b'));
        assert!(contains_char(&list, 'c'));
    }
}