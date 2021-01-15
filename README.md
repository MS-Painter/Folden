# Folden
Full encompassing repository, 

for the client and server pattern elements, 

for the previously named "Fun folder" project

The project structures around 3 separate sub-crates:

- Client

Binary program crate.
Also, root crate of repository.

Responsible for contacting the server side with client requests.

- Server

Binary program crate.

Responsible for the lifetime executable,
 
 of the server which runs in the background.

- Folder Handlers

Library crate.

Responsible for handlers api shared between client and server.

Also, contains common handlers which are available to utilize.


# Differences to other crates
- `notify` - Library used to create cross-platform file watchers.
- `watchexec` - Application used for single file watching cli needs.
- `Folden` - System wide application for file watching needs.

# Contributing
### gRPC auto completed intelliSense
If you're interested in working in an easy to discern environment using IDEA IDE -

You can enable intelliSense for build generated gRPC structures using the following IDE changes:

1. Enter the IDE's “Experimental Features” dialog. Can be done by pressing `Ctrl+Alt+Shift+/`.

2. Enable the `org.rust.cargo.fetch.out.dir` feature.

3. Restart IDE.