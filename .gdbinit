set auto-load safe-path /
target remote :3333
load
monitor arm semihosting enable
break main