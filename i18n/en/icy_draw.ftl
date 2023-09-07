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
menu-paste-as=Paste as
menu-paste-as-new-image=New image
menu-paste-as-brush=Brush
menu-erase=Erase
menu-flipx=Flip X
menu-flipy=Flip Y
menu-justifyleft=Justify Left
menu-justifyright=Justify Right
menu-justifycenter=Center
menu-crop=Crop
menu-justify_line_center=Center Line
menu-justify_line_left=Left Justify Line
menu-justify_line_right=Right Justify Line
menu-insert_row=Insert Row
menu-delete_row=Delete Row
menu-insert_colum=Insert Column
menu-delete_colum=Delete Column
menu-erase_row=Erase Row
menu-erase_row_to_start=Erase Row to Start
menu-erase_row_to_end=Erase Row to End
menu-erase_column=Erase Column
menu-erase_column_to_start=Erase Column to Start
menu-erase_column_to_end=Erase Column to End
menu-scroll_area_up=Scroll Area Up
menu-scroll_area_down=Scroll Area Down
menu-scroll_area_left=Scroll Area Left
menu-scroll_area_right=Scroll Area Right

menu-selection=Selection
menu-select-all=Select All
menu-deselect=Deselect
menu-reference-image=Open Reference Image…
menu-toggle-reference-image=Toggle Reference Image
menu-clear-reference-image=Clear
menu-pick_attribute_under_caret=Use Attribute Under Caret
menu-default_color=Default Color
menu-toggle_color=Switch Foreground/Background

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
tool-custom-brush=Custom brush

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
anchor_layer_tooltip = Anchor layer

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
export-limit-output-line-length-label=Limit output line length
export-maximum_line_length=Maximum line length

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
layer_tool_menu_resize_layer=Resize layer
layer_tool_menu_new_layer=New layer
layer_tool_menu_duplicate_layer=Duplicate layer
layer_tool_menu_merge_layer=Merge layer
layer_tool_menu_delete_layer=Delete layer

char_table_tool_title=Char table
bitfont_tool_title=Fonts
minimap_tool_title=Minimap

no_document_selected=No document selected

undo-draw-ellipse=Draw ellipse
undo-draw-rectangle=Draw rectangle
undo-paint-brush=Paintbrush
undo-pencil=Pencil
undo-eraser=Eraser
undo-bucket-fill=Bucket fill
undo-line=Line
undo-cut=Cut
undo-paste-glyph=Paste glyph
undo-bitfont-flip-y=Flip Y
undo-bitfont-flip-x=Flip X
undo-bitfont-move-down=Move down
undo-bitfont-move-up=Move up
undo-bitfont-move-left=Move left
undo-bitfont-move-right=Move right
undo-bitfont-inverse=Inverse
undo-bitfont-clear=Clear
undo-bitfont-edit=Edit
undo-render_character=Render character
undo-delete_character=Delete character