use my_lisp::Value;
use std::rc::Rc;

fn build_long_list(count: usize) -> Value {
    let mut list = Value::Nil;
    for _ in 0..count {
        list = Value::Pair(Rc::new(Value::Nil), Rc::new(list));
    }
    list
}

#[test]
fn cons_chain_drop_does_not_overflow_stack() {
    let list = build_long_list(150_000);
    drop(list);
}

#[test]
fn cons_chain_clone_does_not_overflow_stack() {
    let list = build_long_list(150_000);
    // Value::clone() for Pair clones the Rcs, which is O(1) and stack-safe.
    // Dropping both lists relies on the iterative Drop mechanism.
    let _cloned = list.clone();
}

#[test]
fn shared_tails_do_not_overflow_stack() {
    let tail = build_long_list(150_000);
    let list1 = Value::Pair(Rc::new(Value::Number(1.0)), Rc::new(tail.clone()));
    let list2 = Value::Pair(Rc::new(Value::Number(2.0)), Rc::new(tail));
    
    drop(list1); // Drops list1's head and its Rc to tail. tail's refcount goes from 2 to 1. No iterative drop for tail.
    drop(list2); // Drops list2's head and its Rc to tail. tail's refcount goes from 1 to 0. Iterative drop handles tail.
}

#[test]
fn improper_lists_do_not_overflow_stack() {
    let count = 150_000;
    // Improper list ends in Number(42.0)
    let mut list = Value::Number(42.0);
    for _ in 0..count {
        list = Value::Pair(Rc::new(Value::Nil), Rc::new(list));
    }
    drop(list);
}
