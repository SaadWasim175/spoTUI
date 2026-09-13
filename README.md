## SpoTUI, terminal-based aesthetic music player
SpoTUI is a straightforward, simple music player made in Rust. It scans your Music folder for songs and loads them all in its library, letting you select whichever one you wish to play. 
It also dynamically reads the metadata of the file to extract the album cover and render it in terminal. However, for that to work, you must have a terminal that supports image rendering (Kitty,
Alacritty, Wezterm, Foot terminal etc).

# Notes on design and color 
Currently, SpoTUI works by using your terminal's theme colors to style itself. Its resizable but looks the best in a portrait orientation. Please note, it does require songs to be installed in your Music 
directory for it to load them automatically. It is extremely memory efficient, having a memory footprint of ~40-60MB at most. 

# Some example images from my PC, might look a bit different on your end depending on your terminal configuration
<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/d198fd2e-764f-459c-9cdd-e5bea4441aa7" />
<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/0152e4f4-5cbb-42fd-826e-c3780411a83e" />
<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/8f480962-3e60-464b-a959-88dfc7fbd3c8" />

