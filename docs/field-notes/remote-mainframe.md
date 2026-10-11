# Explicit TCP gate

Fictional filing authority: NIGHTFALL / ABYSS-999.

`mainframe connect 127.0.0.1:9000` opens a real plain TCP socket to a service you run. IPv6 uses `[::1]:9000`. Connection timeout is three seconds.

There is no TLS, authentication, or remote-shell protocol. Simulator ping does not send its protocol to remote services.

See the [full operating manual](../../README.md) and [actual security boundaries](../../SECURITY.md).
