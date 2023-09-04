font-editor-table = Char table 0-{ $length }:

menu-file=File
menu-new=New…
menu-open=Open…
menu-save=Save
menu-edit-sauce=Edit Sauce Info…
menu-set-canvas-size=Set Canvas Size…
menu-close=Close
menu-save-as=Save As…
menu-export=Export…
menu-edit-font-outline=Font Outline…

menu-edit=Edit
menu-undo=Undo
menu-redo=Redo
menu-undo-op=Undo: { $op }
menu-redo-op=Redo: { $op }

menu-cut=Cut
menu-copy=Copy
menu-paste=Paste
menu-erase=Erase
menu-flipx=Flip X
menu-flipy=Flip Y
menu-justifyleft=Justify Left
menu-justifyright=Justify Right
menu-justifycenter=Center
menu-crop=Crop

menu-selection=Selection
menu-select-all=Select All
menu-deselect=Deselect

menu-help=Help
menu-discuss=Discuss
menu-report-bug=Report Bug
menu-about=About…

tool-fg=Fg
tool-bg=Bg
tool-solid=Solid
tool-line=Line
tool-character=Character
tool-shade=Shade
tool-colorize=Colorize
tool-size-label=Size:
tool-half-block=Half block
toolbar-new=New

new-file-title=New File
new-file-width=Width
new-file-height=Height
new-file-ok=Ok
new-file-cancel=Cancel
new-file-create=Create

edit-sauce-title=Edit Sauce Info
edit-sauce-title-label=Title:
edit-sauce-title-label-length=(35 chars)
edit-sauce-author-label=Author:
edit-sauce-author-label-length=(20 chars)
edit-sauce-group-label=Group:
edit-sauce-group-label-length=(20 chars)
edit-sauce-comments-label=Comments
edit-sauce-letter-spacing=Letter spacing:
edit-sauce-aspect-ratio=Aspect ratio:

edit-canvas-size-title=Set Canvas Size
edit-canvas-size-width-label=Width:
edit-canvas-size-height-label=Height:
edit-canvas-size-resize=Resize

toolbar-size = { $colums } Columns x { $rows } Rows
toolbar-position = ({ $column }:{ $line })

add_layer_tooltip = Add new layer
move_layer_up_tooltip = Move layer up
move_layer_down_tooltip = Move layer down
delete_layer_tooltip = Delete layer

glyph-char-label=Char
glyph-font-label=Font

export-title=Export
export-button-title=Export
export-file-label=File name:
export-video-preparation-label=Video Preparation:
export-video-preparation-None=None
export-video-preparation-Clear=Clear Screen
export-video-preparation-Home=Home Cursor
export-utf8-output-label=Modern terminal format (utf8)
export-save-sauce-label=Save sauce info
export-compression-level-label=Compression level
export-compression-level-off=Off
export-compression-level-medium=Medium
export-compression-level-high=High

select-character-title=Select Character

select-outline-style-title=Outline Font Style Type

about-dialog-title=About Icy Draw
about-dialog-heading = Icy Draw
about-dialog-description = 
    Icy Draw is a tool for creating ANSI and ASCII art.
    It is written in Rust and uses the EGUI library.

    Icy Draw is free software, licensed under the Apache 2 license.
    Source code is available at www.github.com/mkrueger/icy_draw
about-dialog-created_by = Created by { $authors }

edit-layer-dialog-title=Layer properties
edit-layer-dialog-name-label=Name:
edit-layer-dialog-is-visible-checkbox=Visible
edit-layer-dialog-is-edit-locked-checkbox=Edit locked
edit-layer-dialog-is-position-locked-checkbox=Position locked
edit-layer-dialog-is-x-offset-label=X offset:
edit-layer-dialog-is-y-offset-label=Y offset:
edit-layer-dialog-has-alpha-checkbox=Has alpha
edit-layer-dialog-is-alpha-locked-checkbox=Alpha locked

error-load-file=Error loading file: { $error }


select-font-dialog-title=Select Font ({ $fontcount} available)
select-font-dialog-select=Select
select-font-dialog-filter-text=Filter fonts
select-font-dialog-no-fonts=No fonts matches the filter
select-font-dialog-no-fonts-installed=No fonts installed

layer_tool_title=Layer
layer_tool_menu_layer_properties=Layer properties
layer_tool_menu_new_layer=New layer
layer_tool_menu_duplicate_layer=Duplicate layer
layer_tool_menu_merge_layer=Merge layer
layer_tool_menu_delete_layer=Delete layer

char_table_tool_title=Char table
bitfont_tool_title=Fonts

no_document_selected=No document selected

undo-draw-ellipse=Draw ellipse
undo-draw-rectangle=Draw rectangle
undo-paint-brush=Paintbrush
undo-eraser=Eraser
undo-bucket-fill=Bucket fill
undo-line=Line
undo-cut=Cut