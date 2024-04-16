# FAQ, Tips and Tricks

## Can garden print out each command in a multi-line command as it is run?

While `garden -vv` can be used to make `garden` print out the rendered command
right before garden runs it, all of the commands in a multi-line command are
printed at once before any command is run.

You can enable "echo mode" in the command's shell to make it print out
each command as it is run.

Enable echo mode by calling `set -x` in your command.

```yaml
commands:
  echo: |
    set -x
    echo hello
    echo world
```

Calling `set -x` results in the following output from `garden echo`:

```bash
$ garden --quiet echo
+ echo hello
hello
+ echo world
world
```

Lines starting with `+` display the command being used.

The following is printed instead when "echo mode" is not enabled.

```bash
$ garden --quiet echo
hello
world
```
