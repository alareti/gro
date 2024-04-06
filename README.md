# groat

Green reducer-objects-as-thread's

The aim is to define a concurrency model expressive enough to allow synchronous
code define highly concurrent applications. Using only primitives from Rust's
std library, this library essentially relies upon OS runtime support to
enable concurrency as opposed to using runtime libraries like tokio or
async_std.
