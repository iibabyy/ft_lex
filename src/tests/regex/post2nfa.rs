use crate::regex::*;

#[cfg(test)]
mod self_ptr_deep_clone_with_memo_iterative_tests {
    use super::*;
    use std::{collections::HashMap, rc::Rc, time::{Duration, Instant}};

    // Helper function to create a simple linear chain of states
    fn create_linear_chain(length: usize) -> StatePtr {
        let match_state = State::match_();
        
        if length == 0 {
            return match_state;
        }
        
        // Start from the end (match state) and build backwards
        let mut current = match_state;
        
        for i in (0..length).rev() {
            // Calculate character based on position
            let char_val = (b'a' + (i % 26) as u8) as char;
            
            // Create new basic state
            let new_state = State::basic(RegexType::Char(char_val));
            
            // Connect the new state to the current chain
            new_state.borrow_mut().into_basic().unwrap().out.replace(current);
            
            // Update current to be the new head of the chain
            current = new_state;
        }
        
        current
    }

    // Helper function to create a deeply nested split structure
    fn create_nested_split(depth: usize) -> StatePtr {
        // Base case
        if depth == 0 {
            return State::match_();
        }
        
        // Start with a match state at the bottom
        let mut current = State::match_();
        
        // Build the structure iteratively from bottom to top
        for i in 1..=depth {
            let level = depth - i; // Count backwards for character assignment
            let char_val = (b'a' + (level % 26) as u8) as char;
            
            // Create a basic state for this level
            let left = State::basic(RegexType::Char(char_val));
            
            // Connect the basic state to the current structure
            left.borrow_mut().into_basic().unwrap().out.replace(current.clone());
            
            // Create a split that branches to the basic state and the current structure
            current = State::split(left, current);
        }
        
        current
    }

    #[test]
    fn test_iterative_clone_basic_state() {
        // Create a basic state
        let original = State::basic(RegexType::Char('a'));
        let match_state = State::match_();
        original.borrow_mut().into_basic().unwrap().out.replace(match_state);

        // Clone using iterative method
        let (cloned, ptr_list) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());

        // Verify structure
        assert!(State::is_basic_ptr(&cloned));
		let borrow = cloned.borrow();
        let basic = borrow.into_basic().unwrap();
        assert_eq!(basic.c.char().unwrap(), 'a');
        
        // Verify the out pointer points to a match state
        let out = basic.out.borrow();
        assert!(State::is_match_ptr(&out));
    }

    #[test]
    fn test_iterative_clone_split_state() {
        // Create a split state with two basic states
        let left = State::basic(RegexType::Char('a'));
        let right = State::basic(RegexType::Char('b'));
        let split = State::split(left, right);

        // Clone using iterative method
        let (cloned, _) = split.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());

        // Verify structure
        assert!(State::is_split_ptr(&cloned));
		let borrow = cloned.borrow();
        let split_state = borrow.into_split().unwrap();
        
        // Verify left branch
        let left_out = split_state.out1.borrow();
        assert!(State::is_basic_ptr(&left_out));
        assert_eq!(left_out.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
        
        // Verify right branch
        let right_out = split_state.out2.borrow();
        assert!(State::is_basic_ptr(&right_out));
        assert_eq!(right_out.borrow().into_basic().unwrap().c.char().unwrap(), 'b');
    }

    #[test]
    fn test_iterative_clone_deep_structure() {
        // Create a deep structure that would likely cause stack overflow with recursion
        let original = create_linear_chain(1000);
        
        // Clone using iterative method
        let (cloned, _) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());
        
        // Verify the first state
        assert!(State::is_basic_ptr(&cloned));
        assert_eq!(cloned.borrow().into_basic().unwrap().c.char().unwrap(), 'a');
        
        // Follow the chain and verify the last state is a match state
        let mut current = cloned;
        let mut count = 0;
        
        while !State::is_match_ptr(&current) && count < 1100 {
			let tmp = current;
			let borrow = tmp.borrow();
            if let Some(basic) = borrow.into_basic() {
                current = basic.out.borrow().clone();
                count += 1;
            } else {
                panic!("Expected basic state in chain");
            }
        }
        
        assert!(State::is_match_ptr(&current), "Last state should be a match state");
        assert_eq!(count, 1000, "Chain should have 1000 states");
    }

    #[test]
    fn test_iterative_clone_nested_splits() {
        // Create a structure with deeply nested splits
        let original = create_nested_split(100);

        // Clone using iterative method
        let (cloned, _) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());
        
        // Verify it's a split
        assert!(State::is_split_ptr(&cloned));
        
        // Walk the structure and count the depth
        fn count_depth(state: StatePtr) -> usize {
            fn count_depth_with_memo(state: StatePtr, memo: &mut HashMap<*const State, usize>) -> usize {
                let raw_ptr = &*state.borrow() as *const State;

                // Check if we've already counted this state
                if let Some(depth) = memo.get(&raw_ptr) {
                    return *depth;
                }
                
                let result = if State::is_match_ptr(&state) {
                    1
                } else if let Some(basic) = state.borrow().into_basic() {
                    let next_depth = count_depth_with_memo(basic.out.borrow().clone(), memo);
                    1 + next_depth
                } else if let Some(split) = state.borrow().into_split() {
                    let left_depth = count_depth_with_memo(split.out1.borrow().clone(), memo);
                    let right_depth = count_depth_with_memo(split.out2.borrow().clone(), memo);
                    std::cmp::max(left_depth, right_depth)
                } else if state.borrow().is_start_of_line() {
                    let out = state.borrow().start_of_line_out().unwrap();
                    let next_depth = count_depth_with_memo(out.borrow().clone(), memo);
                    1 + next_depth
                } else if state.borrow().is_end_of_line() {
                    let out = state.borrow().end_of_line_out().unwrap();
                    let next_depth = count_depth_with_memo(out.borrow().clone(), memo);
                    1 + next_depth
                } else {
                    0
                };

                // Store the result in the memo
                memo.insert(raw_ptr, result);
                result
            }
            
            count_depth_with_memo(state, &mut HashMap::new())
        }
        
        let cloned_depth = count_depth(cloned);
        let original_depth = count_depth(original);

		assert_eq!(original_depth, cloned_depth);

		// 100 basics + 1 match
		assert_eq!(cloned_depth, 101);

    }

    #[test]
    fn test_cloning_with_memo_reuse() {
        // Create a state that's used multiple times
        let shared = State::basic(RegexType::Char('x'));
        
        // Create two states that point to the shared state
        let a = State::basic(RegexType::Char('a'));
        a.borrow_mut().into_basic().unwrap().out.replace(shared.clone());
        
        let b = State::basic(RegexType::Char('b'));
        b.borrow_mut().into_basic().unwrap().out.replace(shared.clone());
        
        // Create a split that uses both
        let original = State::split(a, b);
        
        // Clone using iterative method
        let (cloned, _) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());
        
        // Get the out states from the split
		let borrow = cloned.borrow();
        let split = borrow.into_split().unwrap();
        let out1 = split.out1.borrow().clone();
        let out2 = split.out2.borrow().clone();
        
        // Get their out states (should be the same object)
        let out1_next = out1.borrow().into_basic().unwrap().out.borrow().clone();
        let out2_next = out2.borrow().into_basic().unwrap().out.borrow().clone();
        
        // Verify they're the same object (Rc::ptr_eq)
        assert!(Rc::ptr_eq(&out1_next, &out2_next), 
                "Shared state should be cloned only once");
    }

    #[test]
    fn test_compare_with_recursive_clone() {
        // Create various test structures
        let tests = vec![
            State::basic(RegexType::Char('a')),
            State::match_(),
            State::split(State::basic(RegexType::Char('a')), State::basic(RegexType::Char('b'))),
            create_linear_chain(10),
            create_nested_split(5)
        ];
        
        for original in tests {
            // Clone both ways
            let (recursive_clone, _) = original.borrow().self_ptr_deep_clone_with_memo(&mut HashMap::new());
            let (iterative_clone, _) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());
            
            // Compare structure equality
            assert!(structures_equal(&recursive_clone, &iterative_clone),
                    "Recursive and iterative clones should be structurally identical");
        }
    }
    
    #[test]
    fn test_compare_clone_performance() {
        // Test cases with different sizes and complexities
        let test_cases = vec![
            ("Small linear chain", create_linear_chain(10)),
            ("Medium linear chain", create_linear_chain(100)),
            ("Large linear chain", create_linear_chain(500)),
            ("Large linear chain", create_linear_chain(800)),
            ("Small nested split", create_nested_split(10)),
            ("Medium nested split", create_nested_split(50)),
            ("Large nested split", create_nested_split(100)),
            ("Large+ nested split", create_nested_split(350)),
            // ("Large+ nested split", create_nested_split(400)),
        ];
        println!("Performance comparison between recursive and iterative cloning:");
        println!("{:<20} | {:<15} | {:<15} | {:<10}", "Test Case", "Recursive (ms)", "Iterative (ms)", "Speedup");
        println!("{:-<67}", "");
        
        for (name, original) in test_cases {
            // Measure recursive clone time
            let recursive_start = Instant::now();
            let (recursive_clone, _) = original.borrow().self_ptr_deep_clone_with_memo(&mut HashMap::new());
            let recursive_duration = recursive_start.elapsed();
            
            // Measure iterative clone time
            let iterative_start = Instant::now();
            let (iterative_clone, _) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());
            let iterative_duration = iterative_start.elapsed();
            
            // Calculate speedup
            let speedup = if iterative_duration.as_millis() > 0 {
                recursive_duration.as_millis() as f64 / iterative_duration.as_millis() as f64
            } else {
                f64::NAN
            };
            
            println!(
                "{:<20} | {:<15.2} | {:<15.2} | {:<10.2}x",
                name,
                recursive_duration.as_millis() as f64,
                iterative_duration.as_millis() as f64,
                speedup
            );
            
            // Verify the clones are structurally identical
            // assert!(structures_equal(&recursive_clone, &iterative_clone),
            //         "Clones should be structurally identical for {}", name);
        }
    }
    
    #[test]
    fn test_detailed_performance_analysis() {
        // Number of iterations for more reliable measurements
        const ITERATIONS: usize = 5;
        
        // Test with a large structure that would benefit from iterative approach
        let original = create_linear_chain(500);
        
        let mut recursive_times = Vec::with_capacity(ITERATIONS);
        let mut iterative_times = Vec::with_capacity(ITERATIONS);
        
        for _ in 0..ITERATIONS {
            // Measure recursive clone
            let start = Instant::now();
            let (recursive_clone, _) = original.borrow().self_ptr_deep_clone_with_memo(&mut HashMap::new());
            recursive_times.push(start.elapsed());
            
            // Measure iterative clone
            let start = Instant::now();
            let (iterative_clone, _) = original.borrow().self_ptr_deep_clone_with_memo_iterative(&mut HashMap::new());
            iterative_times.push(start.elapsed());
            
            // Verify equality
            assert!(structures_equal(&recursive_clone, &iterative_clone));
        }
        
        // Calculate average times
        let avg_recursive = recursive_times.iter().sum::<Duration>().as_micros() as f64 / ITERATIONS as f64;
        let avg_iterative = iterative_times.iter().sum::<Duration>().as_micros() as f64 / ITERATIONS as f64;
        
        // Calculate standard deviations
        let std_dev_recursive = (recursive_times.iter()
            .map(|t| {
                let diff = t.as_micros() as f64 - avg_recursive;
                diff * diff
            })
            .sum::<f64>() / ITERATIONS as f64)
            .sqrt();
            
        let std_dev_iterative = (iterative_times.iter()
            .map(|t| {
                let diff = t.as_micros() as f64 - avg_iterative;
                diff * diff
            })
            .sum::<f64>() / ITERATIONS as f64)
            .sqrt();
        
        println!("\nDetailed Performance Analysis (Linear Chain of 500 states, {} iterations):", ITERATIONS);
        println!("Recursive: {:.2} µs (±{:.2} µs)", avg_recursive, std_dev_recursive);
        println!("Iterative: {:.2} µs (±{:.2} µs)", avg_iterative, std_dev_iterative);
        println!("Speedup: {:.2}x", avg_recursive / avg_iterative);
        
        // Verify the iterative method is faster
        assert!(avg_iterative <= avg_recursive, 
                "Expected iterative method to be faster than recursive method");
    }
    
    // Helper function to compare structure equality
    fn structures_equal(a: &StatePtr, b: &StatePtr) -> bool {
        let mut visited = HashMap::new();
        structures_equal_recursive(a, b, &mut visited)
    }
    
    fn structures_equal_recursive(
        a: &StatePtr, 
        b: &StatePtr, 
        visited: &mut HashMap<(*const State, *const State), bool>
    ) -> bool {
        let a_ptr = &*a.borrow() as *const State;
        let b_ptr = &*b.borrow() as *const State;
        
        // If we've already checked this pair, return the cached result
        if let Some(&result) = visited.get(&(a_ptr, b_ptr)) {
            return result;
        }
        
        // We're now visiting this pair - temporarily mark as equal
        visited.insert((a_ptr, b_ptr), true);
        
        let result = match (&*a.borrow(), &*b.borrow()) {
            (State::Basic(a_basic), State::Basic(b_basic)) => {
                // Compare the character
                if a_basic.c.char() != b_basic.c.char() {
                    return false;
                }
                
                // Compare out states
                structures_equal_recursive(
                    &a_basic.out.borrow(), 
                    &b_basic.out.borrow(), 
                    visited
                )
            },
            
            (State::Split(a_split), State::Split(b_split)) => {
                // Compare both branches
                structures_equal_recursive(
                    &a_split.out1.borrow(), 
                    &b_split.out1.borrow(), 
                    visited
                ) && 
                structures_equal_recursive(
                    &a_split.out2.borrow(), 
                    &b_split.out2.borrow(), 
                    visited
                )
            },
            
            (State::StartOfLine { out: a_out }, State::StartOfLine { out: b_out }) => {
                structures_equal_recursive(
                    &a_out.borrow(), 
                    &b_out.borrow(), 
                    visited
                )
            },
            
            (State::EndOfLine { out: a_out }, State::EndOfLine { out: b_out }) => {
                structures_equal_recursive(
                    &a_out.borrow(), 
                    &b_out.borrow(), 
                    visited
                )
            },
            
            (State::Match, State::Match) => true,
            (State::NoMatch, State::NoMatch) => true,
            (State::None, State::None) => true,
            
            // Different types
            _ => false,
        };
        
        // Update the cache with the real result
        visited.insert((a_ptr, b_ptr), result);
        result
    }
}