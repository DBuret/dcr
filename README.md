DCR is a small standalone web app designed to help debugging the environment of orchestred containers, through a very small Rust app.

DCR offers the following endpoints

- **displays its env** and the details of the http request it received
    - usefull to understand what env and request your containers receive
- **healthcheck** that can also be turned in ok or ko state through a simple http call
    - usefull to play with your container orchestrator (kubernetes, nomad...) routing system.
- **logger** to output what you want on the log output
    - usefull to test your log gathering system.
- **version display** with a stamp
    - usefull to follow rolling updates
- **DNS (name to IP) resolver** 
    - usefull to check what your containers are able to resolve

more details (configuration options) are available [in the asciidoc readme](https://github.com/DBuret/dcr/README.adoc)

It's a personnal project aimed at discovering Rust, so it is not polished.
