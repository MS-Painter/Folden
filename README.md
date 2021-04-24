# What is Folden?

Application to handle and modify files in unique scenarios. System wide.

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
\* Alternatively check out the [example pipeline files]() for common use cases

2. Register pipeline to directory:

```cmd
USAGE:
    folden register [FLAGS] [OPTIONS] <FILE> [directory]

ARGS:
    <FILE>         Handler pipeline configuration file
    <directory>
```

3. That's it! You can modify / check state of handler/s:

```cmd
- folden status ...
- folden start ...
- folden stop ...
- folden modify ...
```
