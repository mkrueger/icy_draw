font-editor-table = Char table 0-{ $length }:

unsaved-title=Untitled

menu-file=File
menu-new=New…
menu-open=Open…
menu-open_recent=Open Recent
menu-open_recent_clear=Clear
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
menu-delete=Delete
menu-rename=Rename
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
menu-mirror_mode=Mirror mode
menu-area_operations=Area

menu-selection=Selection
menu-select-all=Select All
menu-select_nothing=Deselect
menu-inverse_selection=Inverse

menu-colors=Colors

menu-view=View
menu-reference-image=Open Reference Image…
menu-toggle-reference-image=Toggle Reference Image
menu-clear-reference-image=Clear
menu-toggle_fullscreen=Fullscreen
menu-zoom=Zoom ({$zoom})
menu-zoom_reset=Revert Zoom
menu-zoom_in=Zoom In
menu-zoom_out=Zoom Out
menu-guides=Guides
menu-raster=Raster
menu-guides-off=Off
menu-zoom-fit_size=Fit Size

menu-pick_attribute_under_caret=Use Attribute Under Caret
menu-default_color=Default Color
menu-toggle_color=Switch Foreground/Background
menu-fonts=Fonts
menu-open_font_manager=Fonts…
menu-open_font_directoy=Open Font Directory…
menu-open_tdf_directoy=Open TDF Directory…
menu-open_palettes_directoy=Open Palettes Directory…

menu-help=Help
menu-discuss=Discuss
menu-open_log_file=Open log file
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

tool-select-label=Selection mode:
tool-select-normal=Rectangle
tool-select-character=Character
tool-select-attribute=Attribute
tool-select-foreground=Foreground
tool-select-background=Background
tool-select-description=Hold shift to add to a selection. Control/Cmd to remove.

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
edit-canvas-size-resize_layers-label=Resize layers

toolbar-size = { $colums ->
     [1] 1 Column
*[other] { $colums } Columns
} x { $rows ->
     [1] 1 Row
*[other] { $rows } Rows
}

toolbar-position = Ln { $line }, Col { $column }

add_layer_tooltip = Add new layer
move_layer_up_tooltip = Move layer up
move_layer_down_tooltip = Move layer down
delete_layer_tooltip = Delete layer
anchor_layer_tooltip = Anchor layer

glyph-char-label=Char
glyph-font-label=Font

color-dos=DOS
color-ext=EXT
color-custom=USR

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
select-font-dialog-color-font=COLOR
select-font-dialog-block-font=BLOCK
select-font-dialog-outline-font=OUTLINE
select-font-dialog-preview-text=HELLO

layer_tool_title=Layers
layer_tool_menu_layer_properties=Layer properties
layer_tool_menu_resize_layer=Resize layer
layer_tool_menu_new_layer=New layer
layer_tool_menu_duplicate_layer=Duplicate layer
layer_tool_menu_merge_layer=Merge layer
layer_tool_menu_delete_layer=Delete layer
layer_tool_menu_clear_layer=Clear layer

font_tool_select_outline_button=Set Font Outline
font_tool_current_font_label=Current TDF Font
font_tool_no_font=<none>
font_tool_no_fonts_label=
    No tdf fonts found.
    Install new fonts in the font directory
font_tool_open_directory_button=Open font directory

char_table_tool_title=Char table
minimap_tool_title=Preview

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
undo-select=Select

font_manager-builtin_font=BUILTIN
font_manager-library_font=LIBRARY
font_manager-file_font=FILE

autosave-dialog-title=Autosave
autosave-dialog-description=Found an autosave for this file.
autosave-dialog-question=Do you want to use the original file, or load the autosave?
autosave-dialog-load_autosave_button=Load from autosave
autosave-dialog-discard_autosave_button=Discard autosave

paste_mode-description=You're now in paste mode. Use layer tool to add or anchor the layer.
paste_mode-stamp=Stamp
paste_mode-rotate=Rotate
paste_mode-flipx=Flip X
paste_mode-flipy=Flip Y
paste_mode-transparent=Transparent

ask_close_file_dialog-description=Do you want to save the changes you made to { $filename }?
ask_close_file_dialog-subdescription=Your changes will be lost if you don't save them.
ask_close_file_dialog-dont_save_button=Don't save
ask_close_file_dialog-save_button=Save

tab-context-menu-close=Close
tab-context-menu-close_others=Close others
tab-context-menu-close_all=Close all
tab-context-menu-copy_path=Copy path

font-view-char_label=Char
font-view-ascii_label=ASCII
font-view-font_label=Font