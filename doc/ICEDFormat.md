# Icy Draw file format

I dump the Mystic Draw format completely. Fortunately there are 0 files in this format. 
I learned too much about modern ANSIs to restart.
Let's look.

## Goals

- Every supported format should be represented. Including tundra.
- Be compatible to Sauce/XBin models as much as possible.
- Try to be extensible


## Format
[ID]       4      'iced'
[EOF]      1      EOF Char, usually 0x1A hex
[Checksum] 4      BE_U32 CRC32 checksum for [HEADER] and [BLOCKS]
[HEADER]   11     HEADER
[BLOCKS]   *      BLOCKS
<EOF>

### Header
```
Field      Bytes  Meaning
[VER]      2      BE_U16 u8 Major:u8 Minor - [00:00] atm
[Type]     1      0 - ANSI, 1 - PETSCII, 2 - ATASCII, 3 - VIEWDATA
[Width]    4      BE_U32
[Height]   4      BE_U32

```

### Blocks
Until EOF or End Block - read blocks.

#### End Block
Marks EOF
```
Field      Bytes  Meaning
[1]        1      End of blocks
```

#### SAUCE block (only 1 is valid)
```
Field      Bytes  Meaning
[1]        1      ID == 1
[Title]   35      CP 437 Chars - filled with b' ' SAUCE string
[Author]  20      CP 437 Chars - filled with b' ' SAUCE string
[Group]   20      CP 437 Chars - filled with b' ' SAUCE string
[NUM]      1      number of comments (max 255 - 0 is wasted)
[1]..[n]   n*64   Comment line CP 437 0 Terminated - 64 chars max
```

#### Palette block (only 1 is valid)
```
Field      Bytes  Meaning
[2]        1      ID == 2
[NUM]      4      BE_I32 number of colors (atm only 0xFFFF colors are supported - but it may change)
                  In future (maybe): -1 means no numbers and RGB values are directly stored in the Layer    
[1]..[n]   n*4    U8 r,g,b,a values from 0..255
```

#### Bitfont Font Block
Font is a bit more flexible than in the other formats - however due to sauce the 'font name' is always an option.
So there is no real limit here. Extended fonts just get splitted into 2 256 font blocks.

```
Field      Bytes  Meaning
[4]        1      ID == 3
[Slot]     4      BE_U32 Font slot used
[NameLen]  4      BE_U32 Length of Name
[Name]     *      U8 - UTF8 encoded chars
[Length]   4      BE_U32 Data Length
[Data]     *      Font data as PSF
```

#### Layer
Layers have the option to save some space by using a lower attribute and char length than theoretically need by buffer type.

```
Field      Bytes  Meaning
[5]        1      ID == 5
[Title_Len]4      BE_U32 length of the utf8 title
[Title]    *      U8 - UTF8 encoded chars - Note: May only be 16 chars depending on language.
[Flags]    4      BE_U32
                  Bit 1   : is_visible
                  Bit 2   : edit_locked
                  Bit 3   : position_locked
                  Bit 4   : is_transparent
[X]        4      BE_I32
[Y]        4      BE_I32
[Width]    4      BE_U32
[Height]   4      BE_U32
[DataLen]  8      BE_U64 Length of Data
[Data]     *      Ansi encoded data
```
