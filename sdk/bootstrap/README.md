# Historical bootstrap runtime

Only the published PrismPM 0.2.0 AMD64 binary uses this runner. Current SDK
execution stays native. Native ARM64 acceptance requires a real ARM64 runner;
an AMD64 QEMU diagnostic is not that evidence.
The runner reports its invocation and process architecture, not proof of the
physical host architecture. Native SDK acceptance is checked by the release host.

`runtime.lock.json` pins Debian packages, extracted files, and the original
bootstrap identities. Installation extracts files without executing package
scripts or registering binfmt. SDK inventory binds the complete runtime tree.

The historical executable requires `libgcc_s.so.1` and `libc.so.6`; libc
requires `ld-linux-x86-64.so.2`. The loader and static QEMU have no further
ELF dependencies. Installation checks this closure and retains copyright files.

Authorities: [QEMU user-mode execution](https://www.qemu.org/docs/master/user/main.html),
[Debian QEMU](https://packages.debian.org/bookworm/qemu-user-static),
[glibc](https://packages.debian.org/bookworm/libc6), and
[GCC runtime](https://packages.debian.org/bookworm/libgcc-s1).
The dated Debian snapshot and SHA-256 pins are mandatory; missing or changed
bytes fail closed. No bootstrap evidence schema is changed.
