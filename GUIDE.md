# Basic Usage and Features:

## Folders:

Folders can be created by clicking the "New Folder" button in the top left.
Folders can contain either playlists or sub folders.

Middle-clicking on a folder in the folder list on the left will change the folder list to the clicked folder.

If inside a folder, a back button will appear in the top left under the "New Folder" button.

Left-clicking on folders will display all of their contents (playlists and sub folders).

## Playlists:

Playlists can be created by clicking the "New Playlist" button in the top left.

Right-clicking on playlists in the folder list will bring up a dialog menu in which you can modify various properties.

Left-clicking on a playlist will open it to the main screen.

### Songs:

Songs can be added by the "Add Songs" button which will open a file dialog to add songs.
When adding songs it will make a copy of each added song and save them in /napoleon_amp/songs/.

Once the songs have been selected, there is an option to delete the original copies of the songs.
When songs have been imported, you can filter through the songs.

#### Song Search Filters:

Typing in the search bar at the top left will by default search through all the songs in the current playlist which have
any attribute that matches the query.

In order to search by other attributes a custom search syntax can be used.

To use the custom syntax, prefix the search with "$" and then the attribute name (available attributes are "title", "
artist", "album", and "any").
Followed by a colon and the filter term.

To filter all tracks by "Michael Jackson" the following query can be used:
$artist:michael jackson

It is important to note that case is ignored for all search queries.

### Queue:

When music is playing there will be a queue shown on the right side. Clicking on any of the queued songs will skip to
that song in the queue.
