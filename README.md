[![AUR package](https://repology.org/badge/version-for-repo/aur/apnotes.svg)](https://repology.org/project/apnotes/versions)
[![Lines of Code](https://tokei.rs/b1/github/findus/apnotes)](https://tokei.rs/b1/github/findus/apnotes)
[![CI](https://github.com/findus/apnotes/workflows/tagged-release/badge.svg)](https://github.com/findus/apnotes/actions)

<a href="https://repology.org/project/apnotes/versions">
    <img src="https://repology.org/badge/vertical-allrepos/apnotes.svg" alt="Packaging status" align="right">
</a>

# apnotes (apple-notes-bridge)

<p align="center">
  <img style="width:100%" src="https://raw.githubusercontent.com/findus/NotesManager/master/screencast.gif" alt="animated" />
</p>


Interface for Linux to View, Edit, Create, Backup and Sync Notes from apple devices.
Not everything is implemented yet, and you should duplicate your notes somewhere before using this.
Works only, if you dont store your notes in icloud

Right now this repository consists of 3 Crates:
* The Library itself
* An experimental tui client
* A cli client to interact with the library

# Feature Overview

| Feature           | Original Client       | CLI-UI                    |  CLI-Client    |
|---------------    |-----------------------|---------------------------|----------------|
| Add Notes         | ✔                    | ✔                        |✔ |
| Delete Notes      | ✔                    | ✔                         |✔ |
| Move Notes        | ✔                    | ❌                        |❌|
| Edit Notes        | ✔                    | ✔                         |✔ |
| View Notes        | ✔                    | ✔                         |✔ |
| Sync Notes        | ✔                    | ✔                         |✔ |
| Search Notes      | ✔                    | ✔                         |❔ (Scripting) |
| Merge Notes       | ❌                   | ✔ (Max 2 Versions)        |✔ (Max 2 Versions)  |
