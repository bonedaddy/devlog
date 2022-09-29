# Usage

## 1) create repository

First create a repository for your devlogs:

```sh
$> devlog init
```

By default, devlogs are stored in the directory at `$HOME/devlogs`. You can choose a different directory by setting the `DEVLOG_REPO` environment variable. Examining the repository directory, you'll see a file called `000000001.devlog`. This is your first devlog entry. It's just a text file.

## 2) edit most recent devlog file

To open the most recent devlog file, with your configured editor (note: editor configured via `DEVLOG_EDITOR` environment variable):

```sh
$> devlog edit
```

Specific editor:

```sh
$> export DEVLOG_EDITOR=vim
$> devlog edit
```

## 3) quick overview of recent tasks

To see current tasks grouped by status:

```sh
$> devlog status
```

To see all devlog entries:

```sh
$> devlog tail
```

## 4) move incomplete tasks

To move incomplete tasks to a new devlog file (tasks not marked with `+`):

```sh
$> devlog rollover
```

# Devlog Syntax

The following syntax is used when adding tasks:


```markdown
* Use an asterisk (*) for each task you want to complete today.
^ Use a caret symbol (^) for each task that's in progress.
+ Use a plus sign (+) for tasks you completed
- Use a minus sign (-) for tasks that are blocked.
```

Any line that starts with a `*`, `^`, `+`, or `-` is a task. But your devlog is also a place for free-form thoughts. For example:

```markdown
^ Add method `bar` to class `foo`
    The class is in `lib/utils.rs`.
    The new method is a simple data transformation, so testing should be easy...
    I wonder if I can deprecate method `oldbar` once this is merged?

^ Update library `baz` to version 1.2.3
    Opened the PR, waiting on review.

+ Enable feature flag for cache optimization
    Done!  Checked the system this morning, performance is much better.
As you work, you may realize that some tasks are unnecessary, or maybe you need to add more. That's expected! Just make the changes and keep going.
```

# Misc (copied from `docs/guide.html`)

## extend

#### shell scripts

Devlog is designed to be coupled like garden hose with other command-line tools. This allows you to customize it to your workflow.

For example, on many teams you will send a daily "standup" status report to a Slack channel. Suppose you want to report tasks you completed yesterday, tasks you are working on today, and blocked tasks. A simple shell script suffices:

```bash
#!/usr/bin/env sh
echo "Yesterday:"
devlog status -b 1 -s done  # completed in yesterday's entry
devlog status -s done       # completed in today's entry

echo "Today":
devlog status -s todo       # todo in today's entry

echo "Blocked:"
devlog status -s blocked    # blocked in today's entry
```

As another example, suppose you'd like the status report to automatically highlight tasks by status. If you are using vim and have installed the devlog syntax, then you can simply pipe the status output to vim:

```sh
$> devlog status | vim -R -c 'set filetype=devlog' -
```

That's a lot to type, so you probably want to define an alias in your .bashrc or .zshrc configuration:
```sh
alias dls="devlog status | vim -R -c 'set filetype=devlog' -"
```

## hooks

Devlog can be extended through a mechanism called "hooks". A hook is an executable file located in the $DEVLOG_REPO/hooks directory. To enable a hook, make the file executable, like this:

```sh
$> chmod +x $DEVLOG_REPO/hooks/before-edit
```

The following hooks are available:

| Hook	| Invoked By	| When	| Arguments |
|-------|---------------|-------|-----------|
| before-edit	| devlog edit |	Before opening the most recent devlog file in the editor.	| Absolute path of the devlog file. |
| after-edit	| devlog edit | 	After the editor program exits with a successful status code.	| Absolute path of the devlog file. |
| before-rollover |	devlog rollover |	Before creating the new devlog file.	|Absolute path of the latest devlog file before rollover occurs. |
| after-rollover |	devlog rollover	|  After creating the new devlog file. |	The first argument is the absolute path of the old devlog file; the second argument is the absolute path of the newly-created devlog file. |


Hooks provide a flexible mechanism for integrating devlog with other command-line tools. For example, suppose you want to automatically commit your devlog entries to a git repository. One way to achieve this:

Create an after-edit hook to stage the changes in git:
```bash
#!/usr/bin/env sh
set -e
repo="$(dirname $(dirname $0))"
git -C $repo add "$1"
```

Create an after-rollover hook to commit and push the changes to a remote git repository:

```bash
#!/usr/bin/env sh
set -e
repo="$(dirname $(dirname $0))"
git -C $repo add $1
git -C $repo add $2
git -C $repo commit -m "Rollover to $(basename $2)"
git -C $repo fetch
git -C $repo rebase origin/master
git -C $repo push
```

## library
Devlog is available as a Rust library. Using the library, you can access and parse devlog entries. Please see the library documentation for details.