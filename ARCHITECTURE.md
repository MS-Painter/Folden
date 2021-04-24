# Roles

- Client: Cli tool to contact the background service

- Foldend: Background server service.

Stores configuration and mapping of handlers.

Foldend instantiates pipeline handler threads and is responsible for their lifetime.

- Pipeline handler: File watching handler responsible for executing pipeline logic on it's directory.

# Foldend lifecycle

1. Read configuration

2. Startup registered handlers configured to start alongside service startup.

3. Listen for client requests

# Foldend handler thread lifecycle

1. Start file watcher thread on directory.

2. (Optional based on configuration)  Apply pipeline on all files in directory.

3. Read file watcher events to decide on if to execute pipeline.

4. Pipeline execution -

Each `action` is applied sequentially, and is required to succeed,

To advance to the next `action`. Otherwise ending the pipeline for the current event.

File watching events are handled sequentially as well to not apply pipeline on same file.

# Features & Core concepts

- `Event` - Ruleset on what file watching events to apply pipeline on.
- `Action` - Common logic applied as a stage in a pipeline.
- `Input` - References file paths relevant to a single pipeline:
  - `EventFilePath` - File path of the original file the event was referring to.
  - `ActionFilePath` - File path of the previous file that an action digested.

    Some `actions` create / move the original file;

    This fields value will change at the end of every `action` as part of the pipeline.

    Can't be used as on the first action in a pipeline.
- Input file path formatting on specific action fields.
- Datetime formatting on specific action fields using [strftime conventions](https://docs.rs/chrono/latest/chrono/format/strftime/).
