use num::{one, Integer};
use std::collections::HashMap;
use std::hash::Hash;

pub fn find<T: Integer + Copy + Hash>(x: T) -> Vec<HashMap<T, u64>> {
    let addends = find_all_addends(x);

    let mut all_addends = Vec::new();
    for numbers in addends {
        let mut associated = HashMap::<T, u64>::new();
        for number in numbers {
            let current = associated.get(&number).unwrap_or(&0) + 1;
            associated.insert(number, current);
        }
        all_addends.push(associated);
    }

    all_addends
}

fn find_all_addends<T: Integer + Copy>(x: T) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = Vec::new();
    let mut current: Vec<T> = Vec::new();
    backtrack(one(), x, &mut current, &mut result);

    result
}

fn backtrack<T: Integer + Copy>(
    start: T,
    target: T,
    current: &mut Vec<T>,
    result: &mut Vec<Vec<T>>,
) {
    if target == T::zero() {
        result.push(current.clone());
        return;
    }

    let mut i = start;
    while i <= target {
        if i > target {
            break;
        }
        current.push(i);
        backtrack(i, target - i, current, result);
        current.pop();
        i = i + one();
    }
}
