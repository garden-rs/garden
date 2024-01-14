# Python Commands

This example demonstrates how to configure `garden.shell` to use `python3`
when running custom commands.

## Examples

```bash
garden hello
garden info
# Pass additional arguments after the double-dash `--` end-of-options marker.
garden hello -- cat
```

## garden.yaml

```yaml
{{#include python/garden.yaml}}
```
