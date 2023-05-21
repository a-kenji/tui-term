

# How to 
Make sure that cols and rows are (80, 24)
```
tput lines;tput cols
```
Record your session with
```
script
```

The recorded `script` session should have the correct lines and cols specified:
```
Script started on *** [TERM="tmux-256color" TTY="/dev/pts/3" COLUMNS="80" LINES="24"]
```

# References
- [Wez Csi References](https://wezfurlong.org/wezterm/escape-sequences.html)
- [fnky ASNI Escape Codes Gist](https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797)
- [VT100 Escape Codes](https://espterm.github.io/docs/VT100%20escape%20codes.html)
- [DEC ANSI Parser](https://vt100.net/emu/dec_ansi_parser)
- [DEC ANSI Parameters](https://vt100.net/docs/vt100-ug/chapter3.html)
