Output demo.gif

# Here are a few hacks, to make this demo feel like
# it has a somewhat consistent framerate

#Set Framerate 24
Set FontSize 22
Set Width 1200
Set Height 1000
Set Shell fish

Type "echo 'Welcome to tui-term! A pseudoterminal library for ratatui.'"
Sleep 5s
Enter
Sleep 5s

Type "cargo run --example smux --release"
Sleep 2s
Enter
Sleep 10s

Type "echo 'You can find me in examples/smux.rs' | nix run n#lolcat"
Sleep 2s
Enter
Sleep 5s

Ctrl+N
Type "bat ./examples/smux.rs"
Sleep 500ms
Enter
Ctrl+D
Sleep 500ms
Ctrl+D
Sleep 500ms
Ctrl+D
Sleep 500ms
Ctrl+D
Sleep 500ms
Type J
Sleep 500ms
Type J
Sleep 500ms
Type J
Sleep 500ms
Type J
Sleep 500ms
Type J
Sleep 500ms
Type J
Sleep 500ms
Type J
Sleep 500ms
Type J
Sleep 500ms
Type J
Sleep 5s

Ctrl+N
Type "echo 'With me you can run multiple programs in one terminal, and display them in the context of your ratatui app!'"
Sleep 500ms
Enter

Ctrl+K
Sleep 500ms
Enter
Ctrl+K
Sleep 500ms
Enter
Type "echo 'If you have any questions, don't hesitate reaching out!"
Sleep 2s
Enter


Sleep 15s
