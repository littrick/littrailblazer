# Distro Trailblazer

Every time I use a new distribution, I have to reinstall all my scripts, aliases, environment variables, and software from scratch, which is quite tedious. That's why I created this tool.

I made this tool for two main reasons:

To quickly unify my commands, aliases, and scripts across environments like WSL, servers, build hosts, and fresh installations, simplifying the setup process.

Because I really like Rust and wanted to improve my proficiency with it.

The goal of this tool is simple: one command to fully set up a new system environment, such as `sh install.sh`. For an even simpler approach, the files can be hosted on a server and executed via:

```sh
curl https:://file.server.com/install.sh | sh
```

So, this is a system provisioning tool designed for Debian-based distributions.

## Binary Descriptions

- `pioneer`: The main program for provisioning the distro. It handles all software installations, aliases, and configurations.

- `script_builder`: Generates a one-click installation script. This tool bundles pioneer and all configuration files into a single shell script.


## Try It Out

Build the `installer.sh` Script

```sh
cargo r --bin script_builder -- -p pioneer -c test_example/*.toml
```

Test in a Container

```sh
docker run --rm \
-itv $(realpath .):/ws \
-w /ws ubuntu
```

Then run:

```sh
bash installer.sh
```
