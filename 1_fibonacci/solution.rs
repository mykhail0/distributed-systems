pub struct Fibonacci {
    done: bool,
    a: u128,
    b: u128,
}

impl Fibonacci {
    /// Create new `Fibonacci`.
    pub fn new() -> Fibonacci {
        Fibonacci {
            done: false,
            a: 1,
            b: 0,
        }
    }

    /// Calculate the n-th Fibonacci number.
    ///
    /// This shall not change the state of the iterator.
    /// The calculations shall wrap around at the boundary of u8.
    /// The calculations might be slow (recursive calculations are acceptable).
    pub fn fibonacci(n: usize) -> u8 {
        if n == 0 || n == 1 {
            return n as u8;
        }
        let mut a: u8 = 0;
        let mut b: u8 = 1;
        for _i in 2..=n {
            let t: u8 = u8::wrapping_add(a, b);
            a = b;
            b = t;
        }
        b
    }
}

impl Iterator for Fibonacci {
    type Item = u128;

    /// Calculate the next Fibonacci number.
    ///
    /// The first call to `next()` shall return the 0th Fibonacci number (i.e., `0`).
    /// The calculations shall not overflow and shall not wrap around. If the result
    /// doesn't fit u128, the sequence shall end (the iterator shall return `None`).
    /// The calculations shall be fast (recursive calculations are **un**acceptable).
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match u128::checked_add(self.a, self.b) {
            None => {
                self.done = true;
                Some(self.b)
            }
            Some(t) => {
                self.a = self.b;
                self.b = t;
                Some(self.a)
            }
        }
    }
}
