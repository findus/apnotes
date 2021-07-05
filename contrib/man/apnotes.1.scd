apnotes(1)

# NAME

apnotes - interact with notes that got written on note applications from apple devices.

# SYNOPSIS

*apnotes* [FLAGS...] [SUBCOMMAND] [OPTIONS...]

# FLAGS

*-h, --help*
	Show help message and quit.
*-V, --version*
	Show the version number and quit.

# SUBCOMMANDS

*backup*
	Duplicates current note tree on the imap server (unimplemented right now).
*delete <note>*
	Flags a note as deleted, so that it gets deleted by next sync.
	<note> can either be the notes uuid or subject, if multiple subjects exist
	with the same name the first matching note gets selected.
*edit <note>*
	Edits an existing note.
	<note> can either be the notes uuid or subject, if multiple subjects exist
	with the same name the first matching note gets selected.
*help*
	Prints this message or the help of the given subcommand(s).
*list [--names] [--deleted]*
	Lists all available notes.
	*--deleted:* Only show notes that were flagges for deletion.
	*--names:* Only show the names of the notes (hides the uuid).
*merge <note>*
	Merges unmerged note.
	*<note>* can either be the notes uuid or subject, if multiple subjects exist
	with the same name the first matching note gets selected.
*new [--folder <foldername>] <name>*
	Creates a new note. <name> is the title of the note,
	*[--folder]* specifies the subfolder in which the note should get saved, if it does
	not exists it gets created.
	<note> can either be the notes uuid or subject, if multiple subjects exist
	with the same name the first matching note gets selected.
*print <note>*
	Prints note content-
	<note> can either be the notes uuid or subject, if multiple subjects exist
	with the same name the first matching note gets selected.
*sync*
	Syncs local with remote notes and vice versa.
*undelete <note>*
	Removes deletion flag.
	<note> can either be the notes uuid or subject, if multiple subjects exist
	with the same name the first matching note gets selected.

# DESCRIPTION

apnotes was created to interact with notes that were written on apple devices.
Apart from I-Cloud, by sacrifising some functionality like storing pictures, checkboxes or tables
inside note, the notes apps are also able to store notes inside e-mails via imap. That happens
automatically by creating an imap account and check the "notes" checkbox. These notes are getting
stored as plain html messages.

This cli tool tries to provide the editing functionalities that apple provides on macos and ios,
as well as a merging functionality to unite different versions of the same note, that were edited
on multiple devices simultaneously.

When syncing with the e-mail server, the tool initially downloads all notes and stores them inside
a sqlite database and converts the html body to markdown, so that the notes are editable inside a
normal text editor. *This process is destructive to everything that is not representable in markdown.*

After syncing it is possible to interact with the note database, the resulting changes are getting
stored inside the database. The changes are getting transmitted to the e-mail server upon next
synchronization (*apnotes sync*). The tool also takes care about converting the edited markdown text
back to html.

# CONFIGURATION

configuration file path:

. ~/.config/apnotes/config

# AUTHOR

Philipp Hentschel <philipp@f1ndus.de>

Development Repository <https://github.com/findus/apnotes>

# SEE ALSO

*apnotes*(5)





