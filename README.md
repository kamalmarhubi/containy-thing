# contain.rs -- run commands in a container environment without root

## Why?

Docker and rkt are great container runtimes for production use. However, both
require root privileges to run a container. This makes them non-ideal for steps
in a build process.

contain.rs uses unprivileged [user namespaces][user-ns] to allow non-root users to run
commands in the context of a container image.

[user-ns]: http://man7.org/linux/man-pages/man7/user_namespaces.7.html

## Examples

## Goals and non-goals

Contain.rs has one main goal: allow

Allow running commands in a similar environment as Docker without requiring
root on systems with unprivileged user namespaces. The intention is to enable
hermetic build actions that do not require root priviliges, eg for use in
[Bazel].

[bazel]: http://bazel.io/

Contain.rs has a bunch of anti-goals as well:

- running services in a production setting
- managing long-running processes
- controlling resource usage of containers it creates
