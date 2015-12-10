# contain.rs -- simple containment

## Why?

Docker and rkt are great container runtimes for production use. However, both
require root privileges to run a container. This makes them non-ideal for steps
in a build process.

contain.rs builds on [unprivileged user namespaces][unpriviliged-user-ns] to
allow running commands in the context of a container image as an unprivilged
user.

[unprivileged-user-ns]: TODO

## Examples

```
$ contain.rs -v lol:wat some/docker:image some-cmd with args

## Goals
Allow running commands in a similar environment as Docker without requiring
root on systems with unprivileged user namespaces.

## Non-goals
contain.rs is *not* intended to be used for running services in a production
setting.

contain.rs is *not* a sandbox. While unprivileged user namespaces can be used to
create a sandbox, that is not the aim of contain.rs.
