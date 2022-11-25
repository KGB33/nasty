# `nasty` Notifications

`nasty` Notifications is a command line tool to help create [eww widgets](https://github.com/elkowar/eww).

This is mainly accomplished by providing long running commands that `eww` "listens" to. 
Eww integration examples can be found in my 
[`.dotfiles`](https://github.com/KGB33/.dotfiles/blob/cf8317d8bb4ef33903339199218edba490847441/.config/eww/workspaces.yuck#L1)

See `nasty --help` for cli usage.

# Features
## Freedesktop Notification server

Provides a minimal desktop notification server as defined by the [Freedesktop Desktop Notifications Specification](https://specifications.freedesktop.org/notification-spec/notification-spec-latest.html)

A single client command to close notifications is currently implemented.
For other ways to interact with the server use `notify-send` or `busctl`.

## Window Manager Workspaces

Keeps track of active workspaces. Currently only available for [hyprland](https://github.com/hyprwm/Hyprland)

## Weather 

Uses Mozilla Location Services & Weather.Gov to get the current weather
info (No api keys needed!).

Useful URLs
  - https://location.services.mozilla.com/v1/geolocate?key=geoclue
  - [`api.weather.gov` docs](https://www.weather.gov/documentation/services-web-api)


## Feature Wishlist
Stuff I'll maybe eventually get to (PRs welcome).
  - SwayWM Workspace support
  - Send notification command 
  - Notification "expiration timeout" support.
  - Run as daemon/systemd service



