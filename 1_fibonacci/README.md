# Fibonacci numbers

The task is to implement in Rust calculations of Fibonacci numbers. The first
numbers in the sequence are 0 and 1. We start indexing from zero, so the
implementation shall return:

* 0 -> 0
* 1 -> 1
* 2 -> 1
* 3 -> 2
* …

More specifically, you are to implement a Fibonacci struct. It shall provide:

1. An associated function `fibonacci` which returns the n-th Fibonacci number.
The calculations shall be performed using `u8` integers. To implement wrapping
addition you can use the `wrapping_add()` method of `u8`.
2. An iterator (i.e., the `Iterator` trait) which iterates over the Fibonacci
sequence. The calculations shall be performed using `u128` integers. To
implement checked addition you can use the `checked_add()` method of `u128`.

[`solution.rs`](solution.rs) file is a template for your solution. You are to
implement the missing parts (marked there with the `unimplemented!` macro). Do
not change the interface defined in the template, follow the requirements
specified in the doc comments.

The package includes also a [`main.rs`](main.rs) file, which demonstrates an
exemplary usage of the Fibonacci struct, and some simple tests.
