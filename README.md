# Folden
Full encompassing repository for the client and server pattern elements for the previously named "Fun folder" project

The project structures around 3 separate crates:

- Client

Binary program crate.
Also, root crate of repository.
Responsible for contacting the server side with client requests.

- Server

Binary program crate.
Responsible for the executable which handles the lifetime of the server which runs in the background.

- Folder Handlers

Library crate.
Responsible for handlers api shared between client and server.
Also, contains common handlers which are available to utilize.