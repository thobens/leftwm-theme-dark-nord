#!/usr/bin/env bash

THEME="$HOME/.config/leftwm/themes/current/rofi/powermenu.rasi"

rofi_command="rofi -no-config -theme $THEME"

# Options
shutdown="Shutdown"
reboot="Restart"
lock="Lock"
suspend="Suspend"
hibernate="Hibernate"
logout="Logout"

# Variable passed to rofi
options="$lock\n$suspend\n$hibernate\n$logout\n$reboot\n$shutdown"

chosen="$(echo -e "$options" | $rofi_command -dmenu -selected-row 0)"
case $chosen in
    $shutdown)
        systemctl poweroff
        ;;
    $reboot)
		systemctl reboot
        ;;
    $lock)
		if [[ -f /usr/bin/i3lock ]]; then
			i3lock
		elif [[ -f /usr/bin/betterlockscreen ]]; then
			betterlockscreen -l
		fi
        ;;
    $suspend)
        if [[ -f /usr/bin/i3lock ]]; then
			i3lock
		elif [[ -f /usr/bin/betterlockscreen ]]; then
			betterlockscreen -l
		fi
		systemctl suspend
        ;;
    $hibernate)
        if [[ -f /usr/bin/i3lock ]]; then
			i3lock
		elif [[ -f /usr/bin/betterlockscreen ]]; then
			betterlockscreen -l
		fi
		systemctl hibernate
        ;;
    $logout)
        loginctl kill-session $XDG_SESSION_ID
        ;;
esac
