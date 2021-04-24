# State of the project

The project isn't yet at a stage to be officially regarded as stable (v1.0):

- Missing vital QOL features & optimizations.
- Missing installation for OS's planned to be supported.
- Needs to be tested for a much longer period over all supported OS's.

Do take this in mind if you're considering trying it out!

Check out the [roadmap](https://github.com/STRONG-MAD/Folden/projects/1) to see progress and what's planned.

If you find any issues or have an idea in mind, open an issue :)

# Differences to other rust crates

- `notify`:

Library used to create file watchers. Folden uses it behind the scenes.

- `watchexec`:

CLI tool used to apply a command on a watched file / directory.

- `Folden`:

System wide service and application;

Used for managing multiple file watching command handlers.

Installation comes with a CLI and system service.
