# Dracula Rounded

## Packages

```
DE          : Arch
WM          : Leftwm
terminal    : Alacritty/st
colorscheme : Nord
bar         : eww
launcher    : Rofi
```

## Screenshot

![Desktop](./images/desktop1.png)
![Desktop](./images/sysmenu.png)
![Desktop](./images/app_menu.png)
![Desktop](./images/list_menu.png)

## Dependencies

- [leftwm-git](https://github.com/leftwm/leftwm)
- [picom-git](https://github.com/yshui/picom)
- [eww](https://github.com/elkowar/eww)
- [mononoki nerd font](https://github.com/ryanoasis/nerd-fonts) (customized patch)
- [rofi](https://github.com/davatorium/rofi)
- [starship](https://github.com/starship/starship)
- [alacritty](https://github.com/alacritty/alacritty)
- [dunst](https://dunst-project.org)
- [feh](https://github.com/derf/feh)

## Installation

1. Install all required dependencies

    yay -S leftwm playerctl picom-git nerd-fonts-complete rofi alacritty starship eww dunst feh

2. Clone the repository

```BASH
git clone https://github.com/thobens/leftwm-theme-dark-nord.git
```

3. Remove the symlink to your current theme if set

```BASH
rm -f ~/.config/leftwm/themes/current
```
4. Set this as your current theme

```BASH
ln -s /path/to/leftwm-theme-dark-nord ~/.config/leftwm/themes/current
```

5. Restart your window manager

```Default shortcut
$MOD + Shift + r
```

## Configuration

There are two launchers that can be used, list and touch. You can switch between these by linking either to the file launcher.rasi e.g.:
```BASH
ln -s list_launcher.rasi launcher.rasi
```

## Additional Resources

### GTK Theme

https://github.com/EliverLara/Nordic

### GTK Icon Theme

https://store.kde.org/p/1473069/

### VS Code Theme

I personally like the [Nord Native theme](https://marketplace.visualstudio.com/items?itemName=divanvisagie.nord-native-theme)

### Vim Theme and Setup for rust

See https://www.nordtheme.com/docs/ports/vim/installation

```bash
yay -S vim-plug
cp ~/.vimrc ~/.vimrc.bkp
cp ./.vimrc ~/.vimrc
vi ~/.vimrc
```

Then in vim run the commands

```vim
:PlugInstall
:CocInstall coc-rust-analyzer
```

## Credits

The polybar theme is a modified panel created by adi1090x(https://github.com/adi1090x/polybar-themes)

### Wallpapers

background[1-5].jpg: https://vsthemes.org/en/pictures/other/14511-nord-wallpaper-pack.html
background5.png: https://github.com/lokesh-krishna/dotfiles
background6.png: https://www.reddit.com/r/nordtheme/comments/hko43o/underwater_nord_wallpaper_in_8k/
background7.png: https://www.reddit.com/r/MinimalWallpaper/comments/ijwkyd/natural_nord_forest_by_image_go_nord_website/
