[![CI](https://github.com/STRONG-MAD/Folden/actions/workflows/integration.yml/badge.svg)](https://github.com/STRONG-MAD/Folden/actions/workflows/integration.yml)

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
    <path>         File path. Leave empty to generate default name.
```

\* Alternatively check out the [example pipeline files](examples/example_pipelines/execute_make.toml) for common use cases

2. Register pipeline to directory:

```cmd
USAGE:
    folden register [FLAGS] [OPTIONS] <FILE> [directory]

ARGS:
    <FILE>         Handler pipeline configuration file
    <directory>    Directory to register to. Leave empty to apply on current.
```

3. That's it! You can interact with registered handlers (be sure to check out all options using `--help`):

```cmd
folden status ...
folden start ...
folden stop ...
folden modify ...
```

Example interaction - Setting handler to start with service startup:

```cmd
folden modify --startup auto
```

# Learn more

<p align="center">
  <strong>
    <a href="ARCHITECTURE.md">Architecture<a/>&nbsp;&nbsp;&bull;&nbsp;&nbsp;
    <a href="FAQ.md">FAQ<a/>&nbsp;&nbsp;&bull;&nbsp;&nbsp;
    <a href="https://github.com/STRONG-MAD/Folden/releases">Download<a/>
  </strong>
</p>
