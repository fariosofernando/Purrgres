# Multi-instance helpers

Utilities for running Purrgres against several databases in the same
Docker container, each with its own isolated `.purrs` state.

## `run-multi.sh`

Starts one background Purrgres process per database, listed in a config
file (`database:user` per line — see `databases.conf.example`).

```bash
cp databases.conf.example databases.conf
# edit databases.conf with your databases/users

./run-multi.sh \
  --container my-postgres-container \
  --bin /opt/purrgres/purrgres \
  --backup-dir /var/backups/purrgres \
  -c databases.conf
```

All flags can also be set via environment variables (`PURRGRES_CONTAINER`,
`PURRGRES_BIN`, `PURRGRES_BKP_DIR`) — handy for wrapping in your own deploy
scripts.

Each instance:
- runs detached (`nohup`) with its own `$HOME` at `<backup-dir>/<database>/`
- writes its own log to `<backup-dir>/<database>/.purrs/purrgres.log`
- tracks its PID to avoid starting duplicate processes on re-run

Re-running the script is safe: it skips any database whose instance is
already alive.

## `systemd/purrgres@.service`

A systemd template unit for production hosts, as an alternative to
`nohup` — gives you `systemctl status`, `journalctl`, restart-on-failure,
and start-on-boot. See the comments in the unit file for setup steps.
