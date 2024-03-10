# Command Interpreters

This example demonstrates how to use custom interpreters on a per-command basis.

## Examples

```bash
garden hello
garden info
# Pass additional arguments after the double-dash `--` end-of-options marker.
garden hello -- cat
```

## garden.yaml

```yaml
{{#include command-interpreters/garden.yaml}}
```
