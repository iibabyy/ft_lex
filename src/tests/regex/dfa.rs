use crate::regex::*;

#[cfg(test)]
mod iterative_create_tests {
    use super::*;
    use std::collections::{HashMap, HashSet, VecDeque};

    // Helper to create a simple state with a character
    fn create_char_state(c: char) -> StatePtr {
        State::basic(RegexType::Char(c))
    }

    // Helper to create a simple linear NFA: a -> b -> c -> Match
    fn create_linear_nfa(chars: &[char]) -> StateList {
        let mut list = StateList::new();
        if chars.is_empty() {
            list.add_state(&State::match_());
            return list;
        }
        
        let mut current = State::match_();
        for &c in chars.iter().rev() {
            let state = create_char_state(c);
            state.borrow_mut().into_basic().unwrap().out.replace(current);
            current = state;
        }
        
        list.add_state(&current);
        list
    }

    // Helper to create a branching NFA: Split -> (a, b) 
    fn create_branching_nfa(chars: &[char]) -> StateList {
        let mut list = StateList::new();
        if chars.is_empty() {
            list.add_state(&State::match_());
            return list;
        }
        
        let mut branches = Vec::new();
        for &c in chars {
            let state = create_char_state(c);
            let match_state = State::match_();
            state.borrow_mut().into_basic().unwrap().out.replace(match_state);
            branches.push(state);
        }
        
        // Create a binary tree of splits to connect all branches
        while branches.len() > 1 {
            let mut new_branches = Vec::new();
            for chunk in branches.chunks(2) {
                if chunk.len() == 2 {
                    let split = State::split(chunk[0].clone(), chunk[1].clone());
                    new_branches.push(split);
                } else {
                    new_branches.push(chunk[0].clone());
                }
            }
            branches = new_branches;
        }
        
        list.add_state(&branches[0]);
        list
    }

    // Helper to compare DFA structures
    fn dfas_equal(dfa1: &DfaStatePtr, dfa2: &DfaStatePtr) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Start with the two root states
        queue.push_back((dfa1.clone(), dfa2.clone()));
        
        while let Some((state1, state2)) = queue.pop_front() {
            let s1 = state1.borrow();
            let s2 = state2.borrow();
            
            // Check ID equality (not strictly necessary, but helpful)
            if s1.id != s2.id {
                return false;
            }
            
            // Compare states sets
            if s1.states != s2.states {
                return false;
            }
            
            // Compare match states
            if s1.matchs != s2.matchs {
                return false;
            }
            
            // Compare transitions - must have same keys
            if s1.next.keys().collect::<HashSet<_>>() != s2.next.keys().collect::<HashSet<_>>() {
                return false;
            }
            
            // Mark as visited
            let pair_id = (s1.id, s2.id);
            if !visited.insert(pair_id) {
                continue; // Already visited this pair
            }
            
            // Compare next states recursively
            for (input, next_states1) in &s1.next {
                if let Some(next_states2) = s2.next.get(input) {
                    if next_states1 != next_states2 {
                        return false;
                    }
                    
                    // Get the actual DFA states from memory and compare them
                    // Skip if we've already visited this pair
                }
            }
        }
        
        true
    }

    // Count total states in a DFA
    fn count_dfa_states(start: &DfaStatePtr) -> usize {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back(start.clone());
        
        while let Some(state) = queue.pop_front() {
            let state_ref = state.borrow();
            
            if visited.insert(state_ref.id) {
                // Add all next states to the queue
                for (_, next_states) in &state_ref.next {
                    // Find the DFA state for these NFA states
                    // This is tricky without access to the memory HashMap
                    // We need to track the DFA states we've created
                }
            }
        }
        
        visited.len()
    }

    #[test]
    fn test_create_empty_dfa() {
        // Create an empty NFA
        let empty_list = StateList::new();
        
        // Create DFAs using both methods
        let mut memory1 = HashMap::new();
        let recursive_dfa = DfaState::recursive_create(empty_list.clone(), &mut memory1);
        
        let iterative_dfa = DfaState::iterative_create(empty_list);
        
        // Compare the results
        assert_eq!(recursive_dfa.borrow().id, iterative_dfa.borrow().id);
        assert_eq!(recursive_dfa.borrow().states, iterative_dfa.borrow().states);
        assert_eq!(recursive_dfa.borrow().matchs, iterative_dfa.borrow().matchs);
        assert_eq!(
            recursive_dfa.borrow().next.len(), 
            iterative_dfa.borrow().next.len()
        );
    }
    
    #[test]
    fn test_create_simple_dfa() {
        // Create a simple NFA: 'a'
        let nfa_list = create_linear_nfa(&['a']);
        
        // Create DFAs using both methods
        let mut memory1 = HashMap::new();
        let recursive_dfa = DfaState::recursive_create(nfa_list.clone(), &mut memory1);
        
        let iterative_dfa = DfaState::iterative_create(nfa_list);
        
        // Compare the IDs, states, and transitions
        assert_eq!(recursive_dfa.borrow().id, iterative_dfa.borrow().id);
        assert_eq!(recursive_dfa.borrow().states, iterative_dfa.borrow().states);
        
        // Both should have one transition for 'a'
        assert_eq!(recursive_dfa.borrow().next.len(), iterative_dfa.borrow().next.len());
        
        // Verify the transition is for character 'a'
        assert!(recursive_dfa.borrow().next.contains_key(&InputCondition::Char('a')));
        assert!(iterative_dfa.borrow().next.contains_key(&InputCondition::Char('a')));
    }
    
    #[test]
    fn test_linear_dfa() {
        // Create a linear NFA: 'abc'
        let nfa_list = create_linear_nfa(&['a', 'b', 'c']);
        
        // Create DFAs using both methods
        let mut memory1 = HashMap::new();
        let recursive_dfa = DfaState::recursive_create(nfa_list.clone(), &mut memory1);
        
        let iterative_dfa = DfaState::iterative_create(nfa_list);
        
        // Check states and transitions
        assert_eq!(recursive_dfa.borrow().id, iterative_dfa.borrow().id);
        assert_eq!(recursive_dfa.borrow().states, iterative_dfa.borrow().states);
        
        // Should have one transition for 'a'
        assert_eq!(recursive_dfa.borrow().next.len(), iterative_dfa.borrow().next.len());
        assert!(recursive_dfa.borrow().next.contains_key(&InputCondition::Char('a')));
        assert!(iterative_dfa.borrow().next.contains_key(&InputCondition::Char('a')));
    }
    
    #[test]
    fn test_branching_dfa() {
        // Create a branching NFA: (a|b|c)
        let nfa_list = create_branching_nfa(&['a', 'b', 'c']);
        
        // Create DFAs using both methods
        let mut memory1 = HashMap::new();
        let recursive_dfa = DfaState::recursive_create(nfa_list.clone(), &mut memory1);
        
        let iterative_dfa = DfaState::iterative_create(nfa_list);
        
        // Check states and transitions
        assert_eq!(recursive_dfa.borrow().id, iterative_dfa.borrow().id);
        assert_eq!(recursive_dfa.borrow().states, iterative_dfa.borrow().states);
        
        // Should have transitions for 'a', 'b', and 'c'
        assert_eq!(recursive_dfa.borrow().next.len(), iterative_dfa.borrow().next.len());
        assert!(recursive_dfa.borrow().next.contains_key(&InputCondition::Char('a')));
        assert!(recursive_dfa.borrow().next.contains_key(&InputCondition::Char('b')));
        assert!(recursive_dfa.borrow().next.contains_key(&InputCondition::Char('c')));
        
        assert!(iterative_dfa.borrow().next.contains_key(&InputCondition::Char('a')));
        assert!(iterative_dfa.borrow().next.contains_key(&InputCondition::Char('b')));
        assert!(iterative_dfa.borrow().next.contains_key(&InputCondition::Char('c')));
    }
    
    #[test]
    fn test_complex_dfa() {
        // Create a more complex NFA: linear combination of chars
        let long_chars: Vec<char> = ('a'..='z').collect();
        let nfa_list = create_linear_nfa(&long_chars);
        
        // Create DFAs using both methods
        let mut memory1 = HashMap::new();
        let recursive_dfa = DfaState::recursive_create(nfa_list.clone(), &mut memory1);
        
        let iterative_dfa = DfaState::iterative_create(nfa_list);
        
        // Check states and transitions
        assert_eq!(recursive_dfa.borrow().id, iterative_dfa.borrow().id);
        assert_eq!(recursive_dfa.borrow().states, iterative_dfa.borrow().states);
        
        // Should have one transition for 'a'
        assert_eq!(recursive_dfa.borrow().next.len(), iterative_dfa.borrow().next.len());
        assert!(recursive_dfa.borrow().next.contains_key(&InputCondition::Char('a')));
        assert!(iterative_dfa.borrow().next.contains_key(&InputCondition::Char('a')));
    }
    
    #[test]
    fn test_very_complex_dfa() {
        // Test with a branching pattern that would cause stack overflow in recursive version
        let many_chars: Vec<char> = (33..127).map(|i| i as u8 as char).collect();
        let nfa_list = create_branching_nfa(&many_chars);
        
        // Create using iterative method (recursive would likely overflow)
        let iterative_dfa = DfaState::iterative_create(nfa_list);
        
        // Check basic properties
        assert_eq!(iterative_dfa.borrow().id, 0);
        assert!(!iterative_dfa.borrow().states.is_empty());
        
        // Should have transitions for all the characters
        assert_eq!(iterative_dfa.borrow().next.len(), many_chars.len());
        
        // Check a few random transitions
        assert!(iterative_dfa.borrow().next.contains_key(&InputCondition::Char('A')));
        assert!(iterative_dfa.borrow().next.contains_key(&InputCondition::Char('0')));
        assert!(iterative_dfa.borrow().next.contains_key(&InputCondition::Char('z')));
    }
    
    #[test]
    fn test_line_anchors() {
        // Create states with line anchors
        let mut start_list = StateList::new();
        let start_state = State::start_of_line();
        let a_state = create_char_state('a');
        start_state.borrow_mut().start_of_line_out().unwrap().replace(a_state);
        start_list.add_state(&start_state);
        
        let mut end_list = StateList::new();
        let a_state = create_char_state('a');
        let end_state = State::end_of_line();
        a_state.borrow_mut().into_basic().unwrap().out.replace(end_state);
        end_list.add_state(&a_state);
        
        // Create DFAs using both methods
        let mut memory1 = HashMap::new();
        let start_recursive = DfaState::recursive_create(start_list.clone(), &mut memory1);
        let start_iterative = DfaState::iterative_create(start_list);
        
        let mut memory2 = HashMap::new();
        let end_recursive = DfaState::recursive_create(end_list.clone(), &mut memory2);
        let end_iterative = DfaState::iterative_create(end_list);
        
        // Check start of line transitions
        assert!(start_recursive.borrow().next.contains_key(&InputCondition::StartOfLine));
        assert!(start_iterative.borrow().next.contains_key(&InputCondition::StartOfLine));
        
        // Check end of line transitions (in the next state after 'a')
        let start_recursive_borrow = start_recursive.borrow();
		let next_states_recursive = start_recursive_borrow.next.get(&InputCondition::StartOfLine).unwrap();
        let start_iterative_borrow = start_iterative.borrow();
		let next_states_iterative = start_iterative_borrow.next.get(&InputCondition::StartOfLine).unwrap();
        
        assert_eq!(next_states_recursive, next_states_iterative);
        
        // Check end of line anchors
        let end_recursive_borrow = end_recursive.borrow();
		let a_transition_recursive = end_recursive_borrow.next.get(&InputCondition::Char('a')).unwrap();
        let end_iterative_borrow = end_iterative.borrow();
		let a_transition_iterative = end_iterative_borrow.next.get(&InputCondition::Char('a')).unwrap();
        
        // These should have end-of-line transitions
        assert_eq!(a_transition_recursive, a_transition_iterative);
    }
    
    #[test]
    fn test_match_states() {
        // Create an NFA with a match state
        let mut list = StateList::new();
        list.add_state(&State::match_());
        
        // Create DFAs using both methods
        let mut memory1 = HashMap::new();
        let recursive_dfa = DfaState::recursive_create(list.clone(), &mut memory1);
        
        let iterative_dfa = DfaState::iterative_create(list);
        
        // Both should have match states
        assert!(!recursive_dfa.borrow().matchs.is_empty());
        assert!(!iterative_dfa.borrow().matchs.is_empty());
        
        // Both should have the same matches
        assert_eq!(recursive_dfa.borrow().matchs, iterative_dfa.borrow().matchs);
    }
    
    #[test]
    fn test_memory_reuse() {
        // Create a simple pattern that reuses states
        let a_state = create_char_state('a');
        let match_state = State::match_();
        a_state.borrow_mut().into_basic().unwrap().out.replace(match_state.clone());
        
        let b_state = create_char_state('b');
        b_state.borrow_mut().into_basic().unwrap().out.replace(match_state.clone());
        
        let split = State::split(a_state.clone(), b_state.clone());
        
        let mut list = StateList::new();
        list.add_state(&split);
        
        // Create DFAs using both methods
        let mut memory1 = HashMap::new();
        let recursive_dfa = DfaState::recursive_create(list.clone(), &mut memory1);
        
        let iterative_dfa = DfaState::iterative_create(list);
        
        // Both should have the same number of transitions
        assert_eq!(
            recursive_dfa.borrow().next.len(),
            iterative_dfa.borrow().next.len()
        );
        
        // Both should have transitions for 'a' and 'b'
        assert!(recursive_dfa.borrow().next.contains_key(&InputCondition::Char('a')));
        assert!(recursive_dfa.borrow().next.contains_key(&InputCondition::Char('b')));
        
        assert!(iterative_dfa.borrow().next.contains_key(&InputCondition::Char('a')));
        assert!(iterative_dfa.borrow().next.contains_key(&InputCondition::Char('b')));
    }
}