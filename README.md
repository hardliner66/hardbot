# My custom bot

## Info
The bot will not send messages as hardbot. It will send messages as the user given in the TWITCH_NAME, which should be the user you used to authenticate via tmi.

## Setup twitch authentication
You will need to set the following environment variables
- TWITCH_TOKEN can be generated at [twitchapps.com/tmi](https://twitchapps.com/tmi).
- TWITCH_NAME is your twitch name
- TWITCH_CHANNEL the twitch channel to join

## Config
Copy the defaults.toml to the following location:
- Linux: ~/.config/hardbot/config.toml
- Windows: %USERPROFILE%\.config\hardbot\config.toml

change the commands
