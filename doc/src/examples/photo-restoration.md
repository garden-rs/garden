# Photo Restoration

This example demonstrates how to run the
[Bringing Old Photo Back to Life](https://github.com/microsoft/Bringing-Old-Photos-Back-to-Life)
photo restoration project.

## Setup

Run the following commands to clone the repository, download pre-trained deep-learning
model data files used by the software and configure a Python virtualenv used to
run the tools.

*NOTE*: this example uses almost 7GB of disk space.

```bash
# Create a directory we'll clone and build a few repositories.
mkdir -p photo-restoration && cd photo-restoration

# Download and audit the garden file we're going to run.
wget https://raw.githubusercontent.com/davvid/garden/main/doc/src/examples/photo-restoration/garden.yaml
cat garden.yaml

# One-time setup: Clone all of the trees. This will clone an "old-photos" repo.
garden grow old-photos

# One-time setup: Download resources.
garden setup old-photos
```

## Run the Software

Now that everything is setup we can run the tools using the custom `run` command
provided by the `garden.yaml` file. The `run.py` script takes several options.

```bash
garden run old-photos -- --help
```

Arguments can be passed directly to `run.py` by passing additional arguments
after the special double-dash `--` "end of options" marker.

The example above pases the `--help` option for demonstration purposes.
You will have to specify the `--input_folder <folder>` and `--output_folder <folder>`
in order to use the photo restoration tool. See the `--help` output for more details.

## garden.yaml

The following is the contents of the `garden.yaml` file used in this example.

The `setup` command defines what happens during `garden setup old-photos`.

The `run` command defines when happens during `garden run old-photos`.

Additional command-line arguments specified after the double-dash `--` marker are
available to commands via conventional `$1`, `$2`, `$N`, ... `"$@"` shell variables.

```yaml
{{#include photo-restoration/garden.yaml}}
```
