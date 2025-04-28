# /bash/.bashrc に格納
# ~/.bashrc から読み込まれる

PS1='\[\e[1;$(($? == 0 ? 32 : 31))m\]\w >\[\e[0m\] '

alias serial="screen $TARGET_USB 115200"
