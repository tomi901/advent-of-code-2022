use std::ops::{Add, Rem};


pub fn wrap_val<T>(val: T, range: T) -> T
    where T: Rem<Output = T> + PartialOrd + Add<Output = T> + Default + Copy
{
    let wrapped = val % range;
    if wrapped >= Default::default() { wrapped } else { wrapped + range }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use super::*;

    #[rstest]
    #[case(0, 4, 0)]
    #[case(2, 5, 2)]
    #[case(5, 5, 0)]
    #[case(6, 4, 2)]
    #[case(17, 4, 1)]
    #[case(-1, 4, 3)]
    #[case(-7, 4, 1)]
    fn wraps_correctly(
        #[case] val: i32,
        #[case] range: i32,
        #[case] expected: i32,
    ) {
        let result = wrap_val(val, range);
        assert_eq!(result, expected)
    }
}
