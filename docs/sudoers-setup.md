# YAM Sudoers Setup (Ubuntu/Linux)

YAM uses `systemctl` to manage services, which requires `sudo`.
To allow YAM to manage services without a password prompt, configure sudoers:

## Setup

```bash
sudo visudo -f /etc/sudoers.d/yam
```

Add the following (replace `youruser` with your actual username):

```
satoshi ALL=(ALL) NOPASSWD: /usr/bin/systemctl start bitcoind, /usr/bin/systemctl stop bitcoind, /usr/bin/systemctl restart bitcoind
satoshi ALL=(ALL) NOPASSWD: /usr/bin/systemctl start tor, /usr/bin/systemctl stop tor, /usr/bin/systemctl restart tor
satoshi ALL=(ALL) NOPASSWD: /usr/bin/systemctl start electrs, /usr/bin/systemctl stop electrs, /usr/bin/systemctl restart electrs
satoshi ALL=(ALL) NOPASSWD: /usr/bin/systemctl start i2pd, /usr/bin/systemctl stop i2pd, /usr/bin/systemctl restart i2pd
satoshi ALL=(ALL) NOPASSWD: /usr/bin/systemctl start btc-rpc-explorer, /usr/bin/systemctl stop btc-rpc-explorer, /usr/bin/systemctl restart btc-rpc-explorer
```

## Verify

```bash
sudo -n systemctl status bitcoind
```

If no password prompt appears, the setup is correct.
