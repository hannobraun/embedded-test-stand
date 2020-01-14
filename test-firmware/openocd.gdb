target remote | openocd
load
# Required to work around a weird OpenOCD error message when uploading to the
# LPC845-BRK.
monitor reset
continue
