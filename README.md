yak
=======
An *experimental* remote backend for the [jj version control tool](https://jj-vcs.github.io/jj/latest/)

yak is somewhat of a placeholder name.

### Architecture

A high-level overview of how jj-yak is intended to work.

There are three primary pieces of software.

* JJ CLI w/ jj-yak backend
* Daemon
* Remote Backend


1. CLI

The CLI is the primary way an end-user interacts with `jj-yak`. It communicates with the `daemon` over gRPC. The CLI stores no persistent data. It can be used to initiate new jj-yak repositories by requesting the daemon to mount the repo.

```bash
jj yak init bwb@thelastyak.com/repo # initialize a local copy of a repo as bwb
```

2. Daemon

Runs on the end user machine. It is intended to be a long-lived process that is capable of being restarted.
It implements a control interface over gRPC which communicates with the JJ CLI (backend and working copy interfaces). It implements an NFS server and manages the local mounting of repos via an NFS client implementation. It caches reads and writes that interact with the backend.

```bash
jj yak ls # List locally mounted repos
```

3. Backend
Stores all commit and repo data for all users. 
