# What is Folden?

Application to handle and modify files for common scenarios. System wide.

Using simple pipelines, designed to be easily created & monitored from your CLI.

# Motivation

Folden is meant to allow anyone to easily apply common logic to specific directories.

Not recommended currently for production critical or overtly complex operation needs.

# How does it work?

After [installing Folden](https://github.com/STRONG-MAD/Folden/releases), the application service runs in the background.

Use the `folden` command to apply and check directories being handled (as long as the service is up).

# Example usage

1. Create a *new pipeline file (Be sure to modify the file itself accordingly):

```cmd
USAGE:
    folden generate [FLAGS] [OPTIONS] [--] [path]

OPTIONS:
        --actions <actions>...     [possible values: runcmd, movetodir]
        --events <events>...       [possible values: create, modify]

ARGS:
    <path>
```

\* Alternatively check out the [example pipeline files](example_pipelines/execute_make.toml) for common use cases

2. Register pipeline to directory:

```cmd
USAGE:
    folden register [FLAGS] [OPTIONS] <FILE> [directory]

ARGS:
    <FILE>         Handler pipeline configuration file
    <directory>
```

3. That's it! Interact with registered handlers (be sure to check out all options using `--help`):

```cmd
folden status ...
folden start ...
folden stop ...
folden modify ...
```

# Learn more

<p align="center">
  <strong>
    <a href="https://github.com/STRONG-MAD/Folden/blob/317df26966d29ba1a1686b4ac2040d2ebdac272d/ARCHITECTURE.md">Architecture<a/>&nbsp;&nbsp;&bull;&nbsp;&nbsp;
    <a href="https://github.com/STRONG-MAD/Folden/blob/a884734ef2b813b61567cedc1c9cc6c50379934d/FAQ.md">FAQ<a/>&nbsp;&nbsp;&bull;&nbsp;&nbsp;
    <a href="https://github.com/STRONG-MAD/Folden/releases">Download<a/>
  </strong>
</p>
