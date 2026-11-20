# Thread pool

The task is to implement in Rust a thread pool. The thread pool is a structure
that owns a few threads, called workers, and allows of submitting tasks that the
workers execute. Every worker operates in a loop: it waits for a task, executes
it, waits for the next task, and so on. The thread pool manages the number of
workers, and distributes the tasks to the workers. Multiple strategies are
possible, but in the assignment you shall implement arguably the simplest one:

* There is a fixed number of threads, specified when the thread pool is
  constructed.
* The tasks can be executed in any order.

The thread pool shall process tasks concurrently. To synchronize the threads we
suggest using a mutex with a conditional variable.

The pool shall be recognized by Rust as *Sync*. You are not allowed to write any
*unsafe* code (more precisely, you are not allowed to use the `unsafe` keyword).

Also, you shall implement the `Drop` trait for the thread pool. It shall stop
all workers and wait for all threads to finish. It shall also wait until all
submitted tasks are executed.

In the template for the assignment we provide some hints, placing them in
comments and in the `unimplemented!()` macros. However, they are only hints, and
you are not obliged to follow them. You may implement the assignment
differently, as long as you do not modify the public interface.

## Type of the task

In the assignment, a task is a closure of the following type:

```rust
type Task = Box<dyn FnOnce<()> + Send>;
```

This type specifies a closure that shall be run only once (the compiler verifies
this), which is wrapped in a `Box` (so the actual closure is stored on the heap,
and thus the size of a task is known at compile time), and which is marked as
`Send` (the compiler verifies whether it is indeed *Safe*). The `dyn` keyword
means, simplifying, that objects that will be wrapped in the `Box` have to
implement the `FnOnce<()>` and `Send` traits. However, their exact type is not
specified. They can be instances of any type, as long as they implement the
`FnOnce<()>` and `Send` traits (as opposed to, for instance, `Box<String>`,
where objects wrapped in `Box` have to be instances of `String`). Since the
actual type of the objects will be known only at runtime, calls to their methods
have to be dynamically dispatched.
