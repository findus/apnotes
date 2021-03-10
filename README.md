# apple-notes-bridge
Interface for Linux to View, Edit, Create, Backup and Sync Notes from apple devices.
Not everything is implemented yet, and you should duplicate your notes somewhere before using this.
Works only, if you dont store your notes in icloud

Right now this repository consists of 3 Crates:
* The Library itself
* An experimental tui client
* A cli client to interact with the library

# Feature Overview

| Feature        | Original Client       | CLI-UI                    |  CLI-Client    |
|--------------- |-----------------------|---------------------------|----------------|
| Add Notes      | ✔                    | ❌                        |✔ |
| Delete Notes   | ✔                    | ✔                         |✔ |
| Move Notes     | ✔                    | ❌                        |❌|
| Edit Notes     | ✔                    | ✔                         |✔ |
| View Notes     | ✔                    | ✔                         |✔ |
| Sync Notes     | ✔                    | ✔                         |✔ |
| Merge Notes    | ❌                   | ✔ (Max 2 Versions)        |✔ (Max 2 Versions)  |

![](https://raw.githubusercontent.com/findus/NotesManager/master/screencast.gif)
