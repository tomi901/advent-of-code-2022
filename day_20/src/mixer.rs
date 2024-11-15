use std::{cmp::Ordering, collections::HashMap};

use anyhow::Context;

#[derive(Debug, Clone)]
struct Node {
    previous: usize,
    next: usize,
}

#[derive(Debug, Clone)]
pub struct ShiftMixer<'a> {
    original: &'a [i64],
    start: usize,
    map: HashMap<usize, Node>,
}

impl<'a> ShiftMixer<'a> {
    pub fn new(arr: &'a [i64]) -> Self {
        let previous_value = |index: usize| {
            if index > 0 { index - 1 } else { arr.len() - 1 }
        };

        let next_value = |index: usize| {
            if index + 1 < arr.len() { index + 1 } else { 0 }
        };

        let map = arr.iter()
            .enumerate()
            .map(|(i, _)| (i, Node { previous: previous_value(i), next: next_value(i) }))
            .collect();
        Self {
            original: arr,
            start: 0,
            map,
        }
    }

    fn shift_node(&mut self, at: usize) -> Result<(), anyhow::Error> {
        // TODO: Optimization:
        // Instead of shifting X amount of times, we can just remove the node at that position
        // and then move X amount of times and insert the node there, to avoid too many writes.
        // cargo run --release is much faster but we can improve it
        let shift = *self.original.get(at).context("No node found")?;
        match shift.cmp(&0) {
            Ordering::Equal => {},
            Ordering::Less => {
                let mut cur_val = at;
                let move_amount = shift.abs() % (self.original.len() as i64 - 1);
                // println!("Moving {} instead of {}", move_amount, shift.abs());
                for _ in 0..move_amount {
                    let right = cur_val;
                    let left = self.map[&at].previous;
                    self.swap_adjacent(left, right);

                    cur_val = right;
                }
            },
            Ordering::Greater => {
                let mut cur_val = at;
                let move_amount = shift % (self.original.len() as i64 - 1);
                // println!("Moving {} instead of {}", move_amount, shift);
                for _ in 0..move_amount {
                    let left = cur_val;
                    let right = self.map[&at].next;
                    self.swap_adjacent(left, right);
                    if self.start == left {
                        self.start = right;
                    }

                    cur_val = left;
                    // println!("{:?}\n", self.iter().collect::<Vec<_>>());
                }
            },
        };

        Ok(())
    }

    fn swap_adjacent(&mut self, left: usize, right: usize) {
        // Shifting B and C
        // A <-> B <-> C <-> D
        // A <-> C <-> B <-> D
        let right_next = self.map[&right].next;
        let left_previous = self.map[&left].previous;
        // println!("- Swapping: {left_previous} - {left} <-> {right} - {right_next}");

        let left_node = self.map.get_mut(&left).unwrap();
        // let prev_left_node = left_node.clone();
        left_node.previous = right;
        left_node.next = right_next;
        // println!("- Left ({left}) {prev_left_node:?} -> {left_node:?}");
        self.map.get_mut(&right_next).unwrap().previous = left;

        let right_node = self.map.get_mut(&right).unwrap();
        // let prev_right_node = right_node.clone();
        right_node.previous = left_previous;
        right_node.next = left;
        // println!("- Right ({right}) {prev_right_node:?} -> {right_node:?}");
        self.map.get_mut(&left_previous).unwrap().next = right;
    }

    pub fn iter(&self) -> impl Iterator<Item = i64> + '_ {
        let mut val = self.start;
        let mut count = 0;
        std::iter::from_fn(move || {
            if count >= self.original.len() {
                return None;
            }

            let cur = self.original[val];
            val = self.map[&val].next;
            count += 1;
            Some(cur)
        })
    }

    pub fn mix_element(&mut self, index: usize) -> Result<(), anyhow::Error> {
        self.shift_node(index)
    }

    pub fn mix(&mut self) -> Result<(), anyhow::Error> {
        for i in 0..self.original.len() {
            // if i % 500 == 0 {
            //     println!("{}/{}", i + 1, self.original.len());
            // }
            self.mix_element(i)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ShiftMixer;

    #[test]
    fn iter_works_correctly() {
        let initial_arrangement = [1, 2, -3, 3, -2, 0, 4];
        let mixer = ShiftMixer::new(&initial_arrangement);

        let nums = mixer.iter().collect::<Vec<i64>>();

        assert_eq!(&nums, &initial_arrangement);
    }

    #[test]
    fn mixes_correctly_with_example_data() {
        let initial_arrangement = [1, 2, -3, 3, -2, 0, 4];
        let arrangements = [
            [2, 1, -3, 3, -2, 0, 4],
            [1, -3, 2, 3, -2, 0, 4],
            [1, 2, 3, -2, -3, 0, 4],
            [1, 2, -2, -3, 0, 3, 4],
            [1, 2, -3, 0, 3, 4, -2],
            [1, 2, -3, 0, 3, 4, -2],
            [1, 2, -3, 4, 0, 3, -2],
        ];
        let mut mixer = ShiftMixer::new(&initial_arrangement);
        // println!("{mixer:#?}");

        for (i, arrangement) in arrangements.iter().enumerate() {
            mixer.mix_element(i).unwrap();
            let nums = mixer.iter().collect::<Vec<i64>>();
            println!("({}) Comparing {:?} and {:?}", i, nums, arrangement);
            assert_eq!(&nums, arrangement);
        }
    }
}

