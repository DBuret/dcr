DCR is a small standalone web app designed to help debugging the environment of orchestred containers, through a very small Rust app.

DCR offers the following endpoints

- **displays its env** and the details of the http request it received
    - usefull to understand what env and request your containers receive
    - *WIP*: The content of the payload of PUT/POST is not yet included in the display. 
- **healthcheck** that can also be turned in ok or ko state through a simple http call
    - usefull to play with your container orchestrator (kubernetes, nomad...) routing system.
- **logger** to output what you want on the log output
    - usefull to test your log gathering system
    - *WIP:*, your message will be print to log as `b"message"`
- **version display** with a stamp
    - usefull to follow rolling updates
    - *WIP:* stamp display is not clean yet

more details (configuration options) are available at https://github.com/DBuret/dcr

It's a personnal project aimed at discovering Rust, so it is not polished.