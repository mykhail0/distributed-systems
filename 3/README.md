# Event-driven shared-nothing architecture

The task is to implement in Rust a more advanced executor system (although still
not a full-featured one) with two modules calculating the
[Fibonacci sequence](https://en.wikipedia.org/wiki/Fibonacci_number) by
exchanging messages between each other. The system shall consist of one
executor, support a single type of modules (*Fibonnacci*) and a single, but
complex, type of messages (*FibonacciSystemMessage*). It shall support dynamic
module registration (when created, a module is registered in the executor by
itself), it shall support any number of modules (although only two modules make
sense when calculating the Fibonacci sequence this way, technically the executor
system shall be ready to support any number of them), and the executor shall be
run in a separate thread.

To compute the Fibonacci numbers, one module, *A*, shall be initialized with `0`
(the 0th Fibonacci number), and the other one, *B*, with `1` (the 1st Fibonacci
number). The calculation begins when module *B* receives an *Init* message.
Module *B* sends its number to module *A*. Module *A* calculates the 2nd
Fibonacci number, saves it locally, and sends it to module *B*. Module *B*
receives the message, calculates the next Fibonacci number (by adding the
received number to the stored number), saves it locally, and sends it to module
*A*. And so on, back and forth. The process continues until the required index
in the Fibonacci sequence is reached (*FibonacciModules* should track the
current index). When one module notices that the required number is calculated,
it sends a *Done* message to the executor, and the whole program should be
terminated gracefully.

For details about the message types, refer to the provided template. The
template also asks you to include in your implementation a special `println!()`
statement to log progress of the calculations. An exemplary correct output is
provided in [`main.rs`](main.rs). Your solution will also be unit tested, so do
not change the public interface defined in the template.

Solutions that calculate the Fibonacci numbers directly (e.g., without a
separate thread serving the modules, accepting the messages, and delivering them
to the modules) will be discarded.
