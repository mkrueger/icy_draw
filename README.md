# MysticDraw

Back in the 90' I made my own ANSI drawing tool which I used in my BBS for drawing stuff. It supported TheDrawFonts and was a bit more "cool" than TD at that time.
I updated it from time to time - but nobody seemed to use it. Never got a single feedback on that.

Doesn't matter - I know why: Because it was not written in rust :). First I used Turbo Pascal, moved to C 1996? (I think), reworked it in C++ (and introduced a lot of bugs, but with the lack of users it didn't matter).
Now I've translated it to rust. Loading/Saving/Modelling is way better than before and unit tested.

The state of the UI basicall non usuable. Mostly I blame the GTK4 bindings for rust. I know GTK 2.x quite well but for rust the bindings are nearly unusuable in my opnion. So development is a bit slower than I expected.

Note: You can use it as an ansi viewer.
