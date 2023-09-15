# LUA API

Lua is used as scripting language for the animation engine.

## Global variables

| Variable     | Description
|--------------|--------------------------------------
| `cur_frame`  | Number of current frame (1 based)

## Global function

| Function                               | Returns    | Description
|----------------------------------------|------------|--------------------------
| `new_buffer(width: i32, height: i32)`  |  Buffer    | Create new, empty buffer with given get_size
| `load_buffer(file_name: String)`       |  Buffer    | Loads a buffer relatively to the animation file.
| `next_frame(buf: Buffer)`              |  -         | Snapshots the "buf" table as new frame and moves to the next frame.

## Buffers

### Fields

| Field      | Description
|---------------|--------------------------
| `width`       | Gets or sets the width of the buffer
| `height`      | Gets or sets the height of the buffer
| `layer_count` | Gets th  number of layers in the buffer
|  |Caret related fields
| `fg` | Gets or sets current foreground color of the caret (palette index)
| `bg` | Gets or sets current background color of the caret (palette index)
| `layer` | Gets or sets the current layer
| `font_page` | Gets or sets the current font page of the caret
| `x`           | Gets or sets the caret x position
| `y`           | Gets or sets the caret y position

### Methods

| Method                                 | Returns | Description
|----------------------------------------|---------|--------------------------
| `clear()`                              | -       | clears the buffer & resets caret

Layer related methods

| Method                                 | Returns | Description
|----------------------------------------|---------|--------------------------
| `set_layer_position(layer, x, y)`      | -       | Sets the offset of a specific layer to move it
| `get_layer_position(layer)`            | x, y    | Gets the offset of a specific layer to move it
| `set_layer_visible(layer, is_visible)` |  -      | Sets if layer is visible
| `get_layer_visible(layer)`             | bool    | Gets if layer is visible

Input/Output

| Method                                 | Returns | Description
|----------------------------------------|---------|--------------------------
| `fg_rgb(r, g, b)`                      | u32     | Sets the caret fg rgb color, returns color palette number
| `fg_rgb(#rrggbb)`                      | u32     | Sets the caret fg rgb color with html style notation , returns color palette number
| `bg_rgb(r, g, b)`                      | u32     | Sets the caret bg rgb color, returns color palette number
| `bg_rgb(#rrggbb)`                      | u32     | Sets the caret bg rgb color with html style notation , returns color palette number
| `set_char(x, y, ch)`                   | -       | Sets a specific char at a given position (uses caret color)
| `get_char(x, y)`                       | char    | Gets a specific char at a given position
| `get_fg(x, y)`                         | u32     | Gets the foreground at a given positon
| `set_fg(x, y, fg)`                     | -       | Sets a specific foreground at a given layer position
| `get_bg(x, y)`                         | u32     | Gets the background at a given positon
| `set_bg(x, y, bg)`                     | -       | Sets a specific foreground at a given layer position
| `print(string)`                        | -       | Prints a string at caret position, advances caret.
