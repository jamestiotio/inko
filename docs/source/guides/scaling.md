# Tuning the runtime

Inko has various configuration settings you can tune, such as the number threads
to use for running processes. There also exist system wide settings you can (or
might need to) tune for large programs.

## Environment variables

Inko's runtime can be configured using various environment variables. These
variables must be set at run-time, not at compile-time.

!!! info
    "CPU" in the default value column refers to the number of CPU cores. For
    example, if you have 16 cores available, then CPU means 16.

| Variable             | Default | Max       | Purpose
|:---------------------|:--------|:----------|:--------------------
| INKO_PROCESS_THREADS | CPU     | 2^16^ - 1 | The number of OS threads to use for running processes.
| INKO_BACKUP_THREADS  | CPU * 4 | 2^16^ - 1 | The number of OS threads to use for replacing OS threads performing blocking operations.
| INKO_NETPOLL_THREADS  | 1       | 128       | The number of OS threads to use for polling sockets for readiness.
| INKO_STACK_SIZE      | 1048576 | 2^32^ - 1 | The size (in bytes) of each process' stack. Stacks don't grow, so be careful to not set this too low or too high.

## Kernel settings

Depending on how many processes you spawn, files you open or other operations
your program performs, you may need to change certain kernel settings.

### Memory map areas

When spawning a large number of processes, you may run into errors such as this:

```
thread 'proc 3' panicked at 'Failed to set up the stack's guard pages. You may need to increase the number of memory map areas allowed: Os { code: 12, kind: OutOfMemory, message: "Cannot allocate memory" }', rt/src/stack.rs:137:66
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

On Linux this happens when you exceed the limit set in
`/proc/sys/vm/max_map_count`, which typically defaults to 65 530. You can
increase this setting as follows:

```
echo 655300 | sudo tee /proc/sys/vm/max_map_count
```

As for what value to use: start by increasing the default value in steps (e.g.
doubling it), until your program no longer crashes, then gradually decrease it
until you've found an ideal value. You can also just use 655 300, which should
be enough for at least 100 000 processes.
