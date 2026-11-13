---
name: bind-port-zero
description: Let a test bind port 0 and read the port the system assigns instead of binding a fixed port, or record why the test needs that fixed port.
---

# Let the system assign the test's port

A test that runs `TcpListener::bind("127.0.0.1:8080")` shares that port with
every other test and process on the machine. Parallel test threads, a second
run of the suite or an unrelated service then make the bind fail, or the test
talks to a stranger that holds the port.

1. Bind port 0, such as `TcpListener::bind("127.0.0.1:0")` or
   `UdpSocket::bind(("127.0.0.1", 0))`; the system assigns a free port.
2. Read the assigned port with `listener.local_addr()?.port()` and pass it, or
   the whole address, to the code under test.
3. When the code under test chooses a fixed port itself, give it the port or
   address as a parameter or configuration value so the test can pass the
   assigned one.
4. When an external program the test cannot configure needs that fixed port,
   record why as an owned `DECISION:`/`INVARIANT:` reference on the line
   directly above the bind.
5. Rerun the reported lint check and the affected tests.

## Rule binding

The measurement flags every Rust call in test code of a path ending in
`TcpListener::bind` or `UdpSocket::bind` whose argument is a string literal
with a nonzero port after its last colon or a tuple whose second element is a
nonzero integer literal, without a `DECISION:`/`INVARIANT:` reference on the
line directly above it. An address held in a constant or variable, a port
computed at run time and binds in product code are not flagged. Do not weaken
the rule or its selectors merely to pass. Apply complexity-discipline; stop
when the test binds a port the system assigns or an owned reference justifies
the fixed port.
