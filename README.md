Notes Engine
============

The notes engine. Edit `config.toml` to taste, then `cargo run`. All files in the configured base
directory will be served according to the glob patterns in the `access` directory. The default
credentials are `admin` and `password`.

The server will try to render things sensibly. I want to eventually support:

- [X] markdown with math blocks (.md)
- [ ] raw html 
- [X] unformatted text (.txt)
- [ ] 3d models?

This is not intended as a complete solution - it is only possible to create users from the web side.
To manage content you should use an additional software to synchronize directories. Like
[syncthing](https://syncthing.net/), `git`, or even `scp`.
