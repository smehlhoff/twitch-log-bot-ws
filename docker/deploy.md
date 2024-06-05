1. Spin up digital ocean droplet (use docker 1-click on marketplace) and reserve an ip address.

2. Configure A records with DNS provider for the following:

- postgres
- prometheus
- grafana
- ubuntu
- n8n
- metabase

3. Log into droplet and execute `sudo apt update && sudo apt upgrade -y` command.

4. Navigate to home directory and git clone project.

5. Transfer secrets to `.envs/.prod` file.

6. Execute `./docker/ufw.sh` file.

7. Execute `docker compose -f prod.yml up --detach --build` command.

8. Log into ubuntu server.

9. Execute `setup.sh` file.

10. Edit `config_example.json` file and rename to `config.json` file.

11. Execute `nohup ./target/release/twitch-log-bot-ws &` command.

12. Configure grafana, metabase, etc., as necessary.
