DOTCONFIG=~/.config

.PHONY: configure
configure:
	@mkdir -p $(DOTCONFIG)/leftwm
	@cp ./.config/leftwm/config.toml $(DOTCONFIG)/leftwm
	@cp ./.config/starship.toml $(DOTCONFIG)/