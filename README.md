<img width="150px" src="https://github.com/hi-im-aurelio/purrgres/raw/master/static/icone.webp">

# Purrgres - Backup Tool for PostgreSQL

Purrgres is an automated PostgreSQL backup tool, specially tailored for environments using Docker containers. It allows you to perform daily backups of your PostgreSQL database, restore specific backups, and view the history of backups performed. With automatic intervals every 24 hours, Purrgres reduces the manual effort to ensure data security and recovery.

> Purrgres is a play on Postgres, but with a “purr” feel, as if it were
> a kitten taking care of the bank.

## Features

- **Automatic backup**: Performs backups of a PostgreSQL database in an automated way.
- **Restore backups**: You can restore backups directly from the file.
- **List backups**: Displays the list of backups performed, including date and time.
- **Check backup status**: Monitor the status of the running backup process.
- **Stop running backup**: Stop the backup process if necessary.

## Semantic Versioning

This project follows [Semantic Versioning (SemVer)](https://semver.org/lang/en-US/), ensuring that versions are consistent and transparent. The version number is composed of three parts:

- **MAJOR**: Incompatible API changes that may break backwards compatibility.
- **MINOR**: Feature additions in a backwards compatible way.
- **PATCH**: Bug fixes and performance improvements, without significant API changes.

### Example:

- `1.0.0` - First stable release.
- `0.1.0` - New features added in a backwards compatible way.
- `0.1.1` - Bug fixes.

Version transitions will be done according to SemVer principles, and any significant updates will be reflected in the version number.

## How to Use

### Download Binary

Binaries are made available with each release for Linux operating systems, you can download the compiled version of `purrgres` for your operating
system directly from the releases section of the [repository](https://github.com/hi-im-aurelio/purrgres/releases). There is no need to compile the
source code manually if you are a Linux user.

Once downloaded, unpack the file:

```bash
    tar -xvf /<you-download-path>/purrgres*.tar.gz
```

Check for the execution bit:

```bash
    chmod +x purrgres
```

And then execute Purrgres:

```bash
    ./purrgres
```

Include the directory Purrgres is in, in your PATH Variable if you wish to be able to execute it anywhere.

Bash:

```bash
    echo 'export PATH="$PATH":"$HOME/<purrgres-path>"' >> ~/.bashrc
```

Zsh:

```bash
    echo 'export PATH="$PATH":"$HOME/<purrgres-path>"' >> ~/.zshrc
```

### Use via Command Line

1. **List Backups Performed**:
   Displays all backups performed so far.

    ```bash
    ./purrgres --list-purrs
    ```

2. **Restore a Backup**
   To restore a backup from a .sql file, use the --rpurry option.

    ```bash
    ./purrgres --rpurry backup.sql --user <USER> --database <DATABASE> --container <CONTAINER>

    ```

3. **Check Running Backup Status**
   Shows the status of the current backup, if any process is running.

    ```bash
    ./purrgres --stats

    ```

4. **Stop the Backup Process**
   If the backup process is in progress, you can stop it.

    ```bash
    ./purrgres --stop

    ```

## How to Run Purrgres

Purrgres can be run in different ways, depending on how you want the process to be managed.

1. **Direct (foreground) execution**

```bash
./purrgres --user <your-data-owner> --database <your-database-name> --container <your-database-container-name>
```

When you run the command directly, the process will run in the foreground in the terminal. This means that the terminal will be "locked" while the process is running, and you will not be able to use the terminal for other tasks while the backup is being performed. In addition, if you close the terminal or the SSH session (if you are working remotely), the process will be stopped.

### When to use:

- Ideal for quick tests or when you want to monitor the execution
  of the program directly in the terminal.

- Useful when you want to interact with the program and observe log
  messages or results in real time.

2. **Background Execution (with nohup)**

```bash
nohup ./purrgres --user <your-data-owner> --database <your-database-name> --container <your-database-container-name> > /dev/null 2>&1 &
```

Running with nohup allows the program to run in the background. The nohup (no hang-up) command ensures that the process continues to run even if you close the terminal or SSH session. The & operator puts the process in the background, allowing you to continue using the terminal for other tasks. The > /dev/null 2>&1 redirection causes the program's standard output and errors to be discarded (not displayed in the terminal), which is useful when you don't want to see the log messages, but still want the process to continue running in the background.

### When to use:

- Ideal for long or automatic backups that need to be executed continuously, without interfering with your interaction with the terminal.

- Necessary when you want to run the process in the background and continue using the terminal for other activities.

- Useful for running the program on a remote server where you do not want to lose the backup execution if the SSH session is disconnected.

Main Differences

| Execution                | In the Foreground                                                                                                | In Background (with `nohup`)                                                                                                              |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| **Command**              | `./purrgres --user <your-data-owner> --database <your-database-name> --container <your-database-container-name>` | `nohup ./purrgres --user <your-data-owner> --database <your-database-name> --container <your-database-container-name> > /dev/null 2>&1 &` |
| **Behavior**             | The terminal is busy while the process is running.                                                               | The process runs in the background, and the terminal is freed up.                                                                         |
| **Monitoring**           | You will see the output directly in the terminal.                                                                | The output is redirected to `/dev/null` and does not appear in the terminal.                                                              |
| **Interaction**          | You can interact with the program in the terminal.                                                               | The program runs in the background without direct interaction.                                                                            |
| **SSH/Terminal Session** | If the terminal is closed, the process is stopped.                                                               | The process will continue running even if the session is closed.                                                                          |

## How to Compile

If you want to compile the source code yourself, follow these steps:

1. **Clone the repository**

```bash
git clone https://github.com/hi-im-aurelio/purrgres.git
cd purrgres
```

2. **Compile the project**

```bash
cargo build --release
```

The binary will be generated in the target/release/purrgres folder.
You can move it to the desired directory or use the cargo install command
to install it globally.

## Contributions

If you would like to contribute to the project, feel free to open pull
requests or report issues.

Be sure to follow coding best practices and provide
clear descriptions of your changes.

## License

This project is licensed under the [MIT License](./LICENSE) - see the LICENSE file for more details.
