# IcyDraw

IcyDraw is the successor of MysticDraw an Ansi drawing tool I developed 1996 and last updated 2003. Due to lack on feedback I lost interest there.
Now 20 years later I've decided to give it an update and here it is.


IcyDraw is a full features ansi drawing tool doing things a bit different than other tools.

# Features

 * File formats: Ansi, Ascii, Artworx ADF, Avatar, BIN, XBIN, PCBoard, iCE, Tundra Draw 
 * Layer model & transparent chars
   * The model is more like modern graphic tools rather than AcidDraw 
 * Can open multiple files
 * Like Mystic draw full TDF font support, including creating/altering fonts
 * Built in font editor for bit fonts
   * Supports the usage of multiple bit fonts in the same file.
 * Suports full RGB colors
 * Sixel support - just paste an image 
 * Shares the engine with IcyTerm and IcyView so it's 100% compatible
 * Probably I forgot most features :)

# File structure

IcyDraw stores data only in one directory (~/.config/icy_draw on linux) - it doesn't scatter data around.

There are several folders:

| path           |  Description
|----------------|:----------------:|
| data/fonts/    | Stores bit fonts
| data/tdf/      | Stores tdf fonts
| data/palettes/ | Stores palettes
| autosave/      | Autosave data
| settings.json  | IcyDraw settings
| icy_draw.log   | IcyDraw log file

Note that fonts/palettes etc. do not need to be unzipped. Just throw a .zip file in there containing fonts and IcyDraw will pick them up.

# What it can't yet do, but potentially will be implemented

Likeley to be implemented by me:
* Full Unicode support
* Non rectangular selections (select by color/char etc., select ellipse)

Unlikely to be done soon:
* Server capabilities
* PETSCII, ATSCII and Viewdata - the engine can do it (see icy_term) 
  
# Help

Contributions are welcome. But also testing & bug reporting or feature requests.

If you can't/want to contriubte code you can donate via paypal to mkrueger@posteo.de