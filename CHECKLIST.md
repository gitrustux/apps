# Rustica OS Command Implementation Checklist

> **Output Format Requirement**: All commands must output stable, machine-readable format by default (JSON/CSV). Human-readable tables available via `--format=table` or `--human-readable` flag.

---

## 1. Package Management (`app` / `rpg`)

### 1.1 `app` (apt replacement)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `app install` | ⬜ | `-y`, `--assume-yes`, `--download-only`, `--fix-broken` | Install packages |
| `app remove` | ⬜ | `-y`, `--purge`, `--auto-remove` | Remove packages |
| `app update` | ⬜ | | Update package lists |
| `app upgrade` | ⬜ | `-y`, `--with-new-pkgs`, `--auto-remove` | Upgrade packages |
| `app full-upgrade` | ⬜ | `-y` | Full distribution upgrade |
| `app search` | ⬜ | `--names-only`, `--full`, `--installed` | Search packages |
| `app show` | ⬜ | `-a`, `--all-versions` | Show package details |
| `app list` | ⬜ | `--installed`, `--upgradable`, `--all-versions` | List packages (JSON default) |
| `app autoremove` | ⬜ | `-y` | Remove unused packages |
| `app clean` | ⬜ | | Clear package cache |
| `app autoclean` | ⬜ | | Clean obsolete packages |
| `app source` | ⬜ | `--download-only`, `--compile` | Download source |
| `app policy` | ⬜ | | Show package policy |
| `app depends` | ⬜ | `--installed`, `--pre-depends` | Show dependencies |
| `app rdepends` | ⬜ | `--installed`, `--recurse` | Show reverse dependencies |
| `app edit-sources` | ⬜ | | Edit sources list |
| `app changelog` | ⬜ | | Show package changelog |

### 1.2 `app-get` (apt-get wrapper)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `app-get install` | ⬜ | `-y`, `-d`, `-f` | |
| `app-get update` | ⬜ | | |
| `app-get upgrade` | ⬜ | `-y`, `-u` | |
| `app-get dist-upgrade` | ⬜ | `-y` | |
| `app-get remove` | ⬜ | `-y`, `--purge` | |
| `app-get autoremove` | ⬜ | `-y` | |
| `app-get clean` | ⬜ | | |
| `app-get source` | ⬜ | `-b`, `-d` | |
| `app-get download` | ⬜ | | |
| `app-get check` | ⬜ | | |
| `app-get -m --print-uris` | ⬜ | | Print URIs only |

### 1.3 `app-cache` (apt-cache replacement)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `app-cache search` | ⬜ | `--names-only`, `--full` | |
| `app-cache show` | ⬜ | `-a`, `--all-versions` | |
| `app-cache policy` | ⬜ | | |
| `app-cache depends` | ⬜ | `--installed`, `--pre-depends` | |
| `app-cache rdepends` | ⬜ | | |
| `app-cache pkgnames` | ⬜ | | |
| `app-cache stats` | ⬜ | | JSON format |
| `app-cache dump` | ⬜ | | |
| `app-cache dumpavail` | ⬜ | | |
| `app-cache unmet` | ⬜ | | |
| `app-cache showsrc` | ⬜ | | |
| `app-cache madison` | ⬜ | | |

### 1.4 `app-key` (apt-key replacement)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `app-key add` | ⬜ | | Add key |
| `app-key del` | ⬜ | | Delete key |
| `app-key list` | ⬜ | | List keys (JSON default) |
| `app-key finger` | ⬜ | | Show fingerprint |
| `app-key export` | ⬜ | | Export key |
| `app-key adv` | ⬜ | | Advanced operations |
| `app-key update` | ⬜ | | Update keys |
| `app-key net-update` | ⬜ | | Update from network |

### 1.5 `applicate` (aptitude replacement)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `applicate install` | ⬜ | `-y`, `-P` | |
| `applicate remove` | ⬜ | `-y`, `-P` | |
| `applicate update` | ⬜ | | |
| `applicate upgrade` | ⬜ | `-y`, `--safe-upgrade` | |
| `appposite full-upgrade` | ⬜ | `-y` | |
| `applicate search` | ⬜ | `--disable-columns`, `-F` | |
| `applicate show` | ⬜ | | |
| `applicate clean` | ⬜ | | |
| `applicate why` | ⬜ | `--explain` | |
| `applicate why-not` | ⬜ | | |
| `appitate holds` | ⬜ | | Show held packages |

### 1.6 `app-add-repository`

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `app-add-repository` | ⬜ | `-r`, `-y`, `-n` | Add PPA/repo |
| `app-add-repository -r` | ⬜ | | Remove repository |

### 1.7 `app-mark` (apt-mark replacement)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `app-mark auto` | ⬜ | | Mark as auto |
| `app-mark manual` | ⬜ | | Mark as manual |
| `app-mark hold` | ⬜ | | Hold package |
| `app-mark unhold` | ⬜ | | Unhold package |
| `app-mark showhold` | ⬜ | | Show held (JSON) |
| `app-mark showauto` | ⬜ | | Show auto (JSON) |
| `app-mark showmanual` | ⬜ | | Show manual (JSON) |

### 1.8 `app-config` (apt-config replacement)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `app-config dump` | ⬜ | `--format=%s` | Dump config (JSON default) |
| `app-config shell` | ⬜ | | Shell mode |
| `app-config set` | ⬜ | | Set option |
| `app-status` | ⬜ | | Show status |

### 1.9 `app-listchanges`

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `app-listchanges` | ⬜ | `--since`, `--show-all` | Show changelog news |

### 1.10 `rpg` (dpkg replacement)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `rpg -i` / `rpg --install` | ⬜ | `--force-confold`, `--force-confnew` | Install .deb |
| `rpg -r` / `rpg --remove` | ⬜ | `--purge` | Remove package |
| `rpg -P` / `rpg --purge` | ⬜ | | Purge package |
| `rpg -l` / `rpg --list` | ⬜ | | List files in package (JSON) |
| `rpg -L` / `rpg --listfiles` | ⬜ | | List files owned by pkg |
| `rpg -S` / `rpg --search` | ⬜ | | Search for file (JSON) |
| `rpg -s` / `rpg --status` | ⬜ | | Show package status (JSON) |
| `rpg -p` / `rpg --print-avail` | ⬜ | | Show available (JSON) |
| `rpg -C` / `rpg --audit` | ⬜ | | Audit broken packages |
| `rpg --get-selections` | ⬜ | | Get selections (JSON) |
| `rpg --set-selections` | ⬜ | | Set selections |
| `rpg --clear-selections` | ⬜ | | Clear selections |
| `rpg -V` / `rpg --version` | ⬜ | | Show version (JSON: `--format=json`) |
| `rpg --verify` | ⬜ | | Verify package |
| `rpg --configure` | ⬜ | | Configure package |
| `rpg --triggers-only` | ⬜ | | Process triggers |
| `rpg --force-depends` | ⬜ | | Force depends |
| `rpg --ignore-depends` | ⬜ | | Ignore depends |
| `rpg --force-downgrade` | ⬜ | | Allow downgrade |
| `rpg -i` (info) | ⬜ | | Show package info |

### 1.11 `rpg-deb` (dpkg-deb replacement)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `rpg-deb -c` / `--contents` | ⬜ | | List contents (JSON) |
| `rpg-deb -f` / `--field` | ⬜ | | Show field |
| `rpg-deb -W` / `--show` | ⬜ | | Show info (JSON) |
| `rpg-deb -e` / `--control` | ⬜ | | Extract control |
| `rpg-deb -x` / `--extract` | ⬜ | | Extract files |
| `rpg-deb -X` / `--vextract` | ⬜ | | Extract verbose |
| `rpg-deb -R` / `--raw-extract` | ⬜ | | Raw extract |
| `rpg-deb -I` / `--info` | ⬜ | | Show info |
| `rpg-deb -b` / `--build` | ⬜ | | Build .deb |
| `rpg-deb --contents` | ⬜ | | |
| `rpg-deb -f` | ⬜ | | Show field |

### 1.12 `rpg-query` (dpkg-query replacement)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `rpg-query -l` / `--list` | ⬜ | `--format=${Package}` | List packages (JSON default) |
| `rpg-query -W` / `--show` | ⬜ | `--showformat`, `-f` | Show info (JSON default) |
| `rpg-query -S` / `--search` | ⬜ | | Search for file (JSON) |
| `rpg-query -s` / `--status` | ⬜ | | Show status |
| `rpg-query -L` / `--listfiles` | ⬜ | | List files (JSON) |
| `rpg-query -p` / `--print-avail` | ⬜ | | Show available |
| `rpg-query -C` / `--audit` | ⬜ | | Audit (JSON) |
| `rpg-query --compare-versions` | ⬜ | | Compare versions |

### 1.13 `rpg-reconfigure`

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `rpg-reconfigure` | ⬜ | `-p`, `--priority` | Reconfigure package |

### 1.14 `rpg-divert` (dpkg-divert replacement)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `rpg-divert --add` | ⬜ | `--local`, `--rename` | Add diversion |
| `rpg-divert --remove` | ⬜ | | Remove diversion |
| `rpg-divert --list` | ⬜ | | List diversions (JSON) |
| `rpg-divert --truename` | ⬜ | | Show true name |
| `rpg-divert --query` | ⬜ | | Query diversion |

---

## 2. System Information

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `uname` | ⬜ | `-a`, `-s`, `-r`, `-v`, `-m`, `-p`, `-i`, `-o` | JSON: `--json` |
| `uname -a` | ⬜ | | All info |
| `lsb_release` | ⬜ | `-a`, `-i`, `-d`, `-r`, `-c` | JSON: `--json` |
| `hostname` | ⬜ | `-i`, `-I`, `-s`, `-f`, `-d`, `-y` | JSON: `--json` |
| `hostnamectl` | ⬜ | `status`, `set-hostname`, `set-chassis` | JSON: `--json=pretty` |
| `uptime` | ⬜ | `-p`, `--since`, `-s` | JSON: `--json` |
| `free` | ⬜ | `-h`, `-b`, `-k`, `-m`, `-g`, `--si`, `-t`, `-o`, `-l` | JSON: `--json` |
| `vmstat` | ⬜ | `-s`, `-d`, `-D`, `-p`, `-S`, `-m`, `-a`, `-t`, `-w` | JSON: `--json` |
| `iostat` | ⬜ | `-c`, `-d`, `-x`, `-k`, `-m`, `-p`, `-t`, `-z`, `-h` | JSON: `--json` |
| `lsblk` | ⬜ | `-a`, `-b`, `-f`, `-m`, `-d`, `-o`, `-O`, `-P`, `-J`, `-p` | JSON: `-J` / `--json` |
| `lsblk -f` | ⬜ | | Filesystems |
| `lsblk -m` | ⬜ | | Permissions |
| `blkid` | ⬜ | `-o`, `-p`, `-s`, `-t`, `-u`, `-c`, `-L`, `-U` | JSON: `-o export` / `--json` |
| `df` | ⬜ | `-h`, `-H`, `-i`, `-k`, `-l`, `-P`, `-T`, `-x`, `-t`, `-a` | JSON: `--output=json` |
| `du` | ⬜ | `-h`, `-s`, `-c`, `-a`, `-x`, `-d`, `--max-depth`, `--time` | JSON: `--json` |
| `mount` | ⬜ | `-t`, `-a`, `-o`, `-v`, `-f`, `-l`, `-n`, `--bind`, `--move` | JSON: `--json` |
| `umount` | ⬜ | `-f`, `-l`, `-n`, `-r`, `-d`, `-v`, `-a` | |
| `findmnt` | ⬜ | `-a`, `-s`, `-t`, `-o`, `-J`, `-p`, `-r`, `-u`, `-U` | JSON: `-J` |
| `stat` | ⬜ | `-c`, `-f`, `-L`, `-t`, `-Z` | JSON: `-c %` format |
| `watch` | ⬜ | `-n`, `-d`, `-h`, `-t`, `-g`, `-e`, `-x` | |

---

## 3. File Operations

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `ls` | ⬜ | `-l`, `-a`, `-h`, `-R`, `-t`, `-r`, `-S`, `-i`, `-F`, `-p` | JSON: `--json` |
| `ls -l` | ⬜ | | Long format (JSON default) |
| `ls -la` | ⬜ | | All, long |
| `tree` | ⬜ | `-a`, `-d`, `-f`, `-F`, `-h`, `-L`, `-p`, `-s`, `-u`, `-g` | JSON: `-J` |
| `pwd` | ⬜ | `-L`, `-P` | JSON: `--json` |
| `cd` | ⬜ | | Shell built-in |
| `cp` | ⬜ | `-r`, `-a`, `-v`, `-p`, `-n`, `-i`, `-b`, `--backup` | JSON: `--json` (log only) |
| `mv` | ⬜ | `-i`, `-f`, `-n`, `-v`, `-b` | JSON: `--json` (log only) |
| `rm` | ⬜ | `-r`, `-f`, `-i`, `-v`, `-d` | JSON: `--json` (log only) |
| `rmdir` | ⬜ | `-p`, `-v` | |
| `mkdir` | ⬜ | `-p`, `-v`, `-m` | JSON: `--json` (log only) |
| `install` | ⬜ | `-m`, `-o`, `-g`, `-d`, `-v`, `-b` | |
| `file` | ⬜ | `-b`, `-i`, `-z`, `-k`, `-L`, `-s`, `--mime-type` | JSON: `--json` |
| `touch` | ⬜ | `-a`, `-m`, `-c`, `-d`, `-r`, `-t` | |
| `basename` | ⬜ | `-a`, `-s` | JSON: `--json` |
| `dirname` | ⬜ | | JSON: `--json` |
| `realpath` | ⬜ | `-e`, `-m`, `-s`, `-z` | JSON: `--json` |
| `readlink` | ⬜ | `-f`, `-e`, `-m`, `-n`, `-v`, `-z` | JSON: `--json` |

---

## 4. Text Processing

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `cat` | ⬜ | `-n`, `-b`, `-s`, `-E`, `-T`, `-v`, `-e`, `-A` | |
| `less` | ⬜ | `-N`, `-S`, `-F`, `-R`, `-m`, `-M`, `-i`, `--follow-name` | |
| `more` | ⬜ | `-d`, `-l`, `-f`, `-p`, `-c`, `-s`, `-u` | |
| `head` | ⬜ | `-n`, `-c`, `-v`, `-q`, `-z` | JSON: `--json` (line counts) |
| `tail` | ⬜ | `-n`, `-f`, `-F`, `--pid`, `-v`, `-q`, `-z`, `--retry` | JSON: `--json` (follow mode) |
| `nl` | ⬜ | `-b`, `-h`, `-i`, `-l`, `-n`, `-p`, `-s`, `-v`, `-w` | JSON: `--json` |
| `wc` | ⬜ | `-c`, `-l`, `-w`, `-m`, `-L` | JSON: `--json` |
| `sort` | ⬜ | `-b`, `-d`, `-f`, `-g`, `-i`, `-M`, `-h`, `-n`, `-R`, `-r` | JSON: `--json` |
| `sort -u` | ⬜ | | Unique |
| `sort -k` | ⬜ | | Key field |
| `sort -t` | ⬜ | | Field separator |
| `uniq` | ⬜ | `-c`, `-d`, `-D`, `-u`, `-i`, `-z`, `-f`, `-s`, `-w` | JSON: `--json` |
| `cut` | ⬜ | `-b`, `-c`, `-d`, `-f`, `-s`, `--complement`, `--output-delimiter` | |
| `paste` | ⬜ | `-d`, `-s`, `-z` | |
| `tr` | ⬜ | `-c`, `-C`, `-d`, `-s`, `-t` | |
| `column` | ⬜ | `-t`, `-s`, `-o`, `-x`, `-n` | JSON: `--json` |
| `fold` | ⬜ | `-b`, `-s`, `-w` | |
| `fmt` | ⬜ | `-w`, `-p`, `-s`, `-t`, `-c` | |
| `join` | ⬜ | `-a`, `-v`, `-1`, `-2`, `-j`, `-o`, `-t`, `-i` | JSON: `--json` |
| `awk` | ⬜ | `-F`, `-v`, `-f` | |
| `sed` | ⬜ | `-n`, `-e`, `-f`, `-E`, `-r`, `-i`, `-z` | |
| `grep` | ⬜ | `-i`, `-v`, `-c`, `-l`, `-L`, `-n`, `-H`, `-h`, `-o`, `-q`, `-m`, `-A`, `-B`, `-C`, `-R`, `-r`, `-E`, `-F`, `-e`, `-f`, `-x`, `-w`, `-z`, `--color`, `--exclude`, `--exclude-dir`, `--include`, `--binary-files` | JSON: `--json` (matches only) |
| `egrep` | ⬜ | | Extended grep |
| `fgrep` | ⬜ | | Fixed grep |
| `strings` | ⬜ | `-a`, `-f`, `-n`, `-t`, `-e`, `-o` | JSON: `--json` |

---

## 5. File Finding

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `find` | ⬜ | `-name`, `-iname`, `-type`, `-size`, `-mtime`, `-atime`, `-ctime`, `-perm`, `-user`, `-group`, `-uid`, `-gid`, `-empty`, `-executable`, `-readable`, `-writable`, `-depth`, `-maxdepth`, `-mindepth`, `-xdev`, `-x`, `-noleaf`, `-regextype`, `-exec`, `-ok`, `-print`, `-print0`, `-ls`, `-delete`, `-prune` | JSON: `--json` |
| `locate` | ⬜ | `-i`, `-c`, `-e`, `-d`, `-r`, `-w`, `-l`, `-0`, `-S` | JSON: `--json` |
| `updatedb` | ⬜ | `-l`, `-U`, `-o`, `-e`, `-f`, `-q` | |
| `which` | ⬜ | `-a`, `-s` | JSON: `--json` |
| `whereis` | ⬜ | `-b`, `-m`, `-s`, `-u` | JSON: `--json` |
| `xargs` | ⬜ | `-0`, `-a`, `-t`, `-n`, `-P`, `-p`, `-I`, `-i`, `-L`, `-d` | |

---

## 6. Networking

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `ip` | ⬜ | `address`, `route`, `link`, `netns`, `neigh`, `rule` | JSON: `-j` / `-json` |
| `ip addr` | ⬜ | `add`, `del`, `show`, `-a`, `-dev` | JSON: `-j` |
| `ip route` | ⬜ | `add`, `del`, `show`, `get`, `replace` | JSON: `-j` |
| `ip link` | ⬜ | `set`, `show`, `up`, `down`, `-name` | JSON: `-j` |
| `ip neigh` | ⬜ | `show`, `flush`, `add`, `del`, `replace` | JSON: `-j` |
| `ss` | ⬜ | `-t`, `-u`, `-w`, `-x`, `-a`, `-l`, `-p`, `-n`, `-m`, `-o`, `-i`, `-s`, `-4`, `-6`, `-Z` | JSON: `-j` / `-H` |
| `ping` | ⬜ | `-c`, `-i`, `-W`, `-s`, `-t`, `-4`, `-6`, `-O` | JSON: `-O` (summary) |
| `traceroute` | ⬜ | `-4`, `-6`, `-n`, `-w`, `-q`, `-m`, `-p`, `-z` | JSON: `--json` |
| `tracepath` | ⬜ | `-n`, `-4`, `-6`, `-b` | JSON: `--json` |
| `arp` | ⬜ | `-a`, `-d`, `-n`, `-v`, `-i` | JSON: `--json` |
| `arping` | ⬜ | `-c`, `-w`, `-I`, `-s`, `-U` | |
| `tcpdump` | ⬜ | `-i`, `-n`, `-X`, `-A`, `-w`, `-r`, `-v`, `-c`, `-e`, `-S`, `-t` | JSON: `-j` (metadata only) |
| `nmap` | ⬜ | `-sS`, `-sT`, `-O`, `-p`, `-PN`, `-v`, `-iL` | JSON: `-oX -` |
| `netstat` | ⬜ | `-t`, `-u`, `-w`, `-x`, `-a`, `-l`, `-p`, `-n`, `-c`, `-r`, `-g`, `-i`, `-s` | JSON: `--json` |
| `nmcli` | ⬜ | `device`, `connection`, `radio`, `show` | JSON: `-j`, `-f`, `--format=json` |
| `nmtui` | ⬜ | | TUI only |
| `iw` | ⬜ | `dev`, `link`, `wiphy`, `station` | JSON: `-j` |
| `iwconfig` | ⬜ | | |
| `iwlist` | ⬜ | `scan`, `wlan` | |
| `rfkill` | ⬜ | `list`, `block`, `unblock`, `-J` | JSON: `-J` |
| `curl` | ⬜ | `-o`, `-O`, `-I`, `-v`, `-s`, `-w`, `-H`, `-d`, `-X`, `-A`, `-u`, `-x`, `-k`, `--head`, `--get` | JSON: `-w "%{json}"` |
| `wget` | ⬜ | `-O`, `-o`, `-c`, `-r`, `-b`, `-q`, `-v`, `-t` | |
| `ftp` | ⬜ | | |
| `sftp` | ⬜ | `-b`, `-o`, `-P` | Batch mode |
| `scp` | ⬜ | `-r`, `-p`, `-P`, `-i`, `-v` | |
| `ssh` | ⬜ | `-i`, `-p`, `-L`, `-R`, `-D`, `-v`, `-N`, `-f`, `-n`, `-X` | |
| `ssh-keygen` | ⬜ | `-t`, `-b`, `-C`, `-f`, `-l`, `-p`, `-y`, `-e`, `-q` | JSON: `-l -f` |
| `ssh-copy-id` | ⬜ | `-i`, `-f`, `-o` | |
| `telnet` | ⬜ | | |
| `nc` | ⬜ | `-l`, `-p`, `-v`, `-z`, `-w`, `-q`, `-u` | |
| `whois` | ⬜ | `-h`, `-p` | |
| `dig` | ⬜ | `@server`, `+short`, `+json`, `-x`, `-t`, `any`, `ns`, `soa` | JSON: `+json` |
| `nslookup` | ⬜ | `-type`, `-port`, `-debug` | JSON: `-json` |

---

## 7. User Management

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `login` | ⬜ | `-f`, `-h`, `-p` | |
| `logout` | ⬜ | | |
| `whoami` | ⬜ | | JSON: `--json` |
| `id` | ⬜ | `-u`, `-g`, `-G`, `-n`, `-r`, `-z` | JSON: `--json` |
| `groups` | ⬜ | | JSON: `--json` |
| `passwd` | ⬜ | `-d`, `-e`, `-i`, `-k`, `-l`, `-n`, `-S`, `-u`, `-w` | JSON: `-S` |
| `chsh` | ⬜ | `-s`, `-l` | |
| `su` | ⬜ | `-`, `-c`, `-s`, `-l`, `-m` | |
| `sudo` | ⬜ | `-i`, `-u`, `-s`, `-H`, `-E`, `-v` | JSON: `--json` (validate) |
| `newgrp` | ⬜ | | |
| `useradd` | ⬜ | `-m`, `-s`, `-d`, `-g`, `-G`, `-u`, `-k`, `-b` | JSON: `--json` |
| `userdel` | ⬜ | `-r`, `-f` | |
| `usermod` | ⬜ | `-a`, `-G`, `-l`, `-s`, `-d`, `-m`, `-u`, `-L`, `-U` | |
| `groupadd` | ⬜ | `-g`, `-o`, `-r`, `-f` | JSON: `--json` |
| `groupdel` | ⬜ | | |
| `groupmod` | ⬜ | `-g`, `-o`, `-n` | |
| `faillog` | ⬜ | `-a`, `-l`, `-m`, `-r`, `-u` | JSON: `--json` |
| `last` | ⬜ | `-n`, `-a`, `-d`, `-F` | JSON: `--format=json` |
| `lastlog` | ⬜ | `-u`, `-n`, `-b`, `-t` | JSON: `--json` |
| `who` | ⬜ | `-a`, `-b`, `-H`, `-l`, `-m`, `-p`, `-q`, `-r`, `-s`, `-t`, `-T`, `-u`, `-w` | JSON: `--json` |
| `w` | ⬜ | `-h`, `-u`, `-s`, `-f` | JSON: `--json` |

---

## 8. Process Management

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `ps` | ⬜ | `-a`, `-u`, `-x`, `-e`, `-f`, `-l`, `-y`, `-o`, `--sort`, `-p`, `--ppid`, `-t`, `-C` | JSON: `--format=json` |
| `ps aux` | ⬜ | | BSD style |
| `ps -ef` | ⬜ | | UNIX style |
| `top` | ⬜ | `-b`, `-n`, `-d`, `-p`, `-u`, `-U`, `-H` | JSON batch: `-b -n 1` |
| `htop` | ⬜ | | TUI only |
| `atop` | ⬜ | `-b`, `-w`, `-M`, `-P`, `-C`, `-m`, `-A`, `-R` | JSON: `--json` |
| `nice` | ⬜ | `-n`, `--adjustment` | |
| `renice` | ⬜ | `-n`, `-p`, `-u`, `-g` | JSON: `--json` |
| `kill` | ⬜ | `-l`, `-s`, `-SIGNAL` | JSON: `-l` |
| `killall` | ⬜ | `-e`, `-i`, `-q`, `-r`, `-s`, `-u`, `-v` | JSON: `--json` |
| `pkill` | ⬜ | `-f`, `-n`, `-o`, `-P`, `-t`, `-u`, `-x`, `-s` | JSON: `--json` |
| `bg` | ⬜ | | |
| `fg` | ⬜ | | |
| `jobs` | ⬜ | `-l`, `-p`, `-r`, `-s` | JSON: `--json` |
| `time` | ⬜ | `-p`, `-v`, `--format` | JSON: `--format=json` |
| `tload` | ⬜ | | |

---

## 9. Disk/Filesystem

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `mount` | ⬜ | (see System Info) | |
| `umount` | ⬜ | (see System Info) | |
| `lsblk` | ⬜ | (see System Info) | |
| `blkid` | ⬜ | (see System Info) | |
| `findmnt` | ⬜ | (see System Info) | |
| `df` | ⬜ | (see System Info) | |
| `du` | ⬜ | (see System Info) | |
| `fsck` | ⬜ | `-a`, `-A`, `-C`, `-f`, `-M`, `-N`, `-P`, `-r`, `-R`, `-T`, `-V`, `-y` | JSON: `--json` |
| `fsck.ext4` | ⬜ | `-b`, `-c`, `-f`, `-p`, `-y`, `-n`, `-v` | |
| `mkfs` | ⬜ | `-t`, `-c`, `-v`, `-n` | |
| `mkfs.ext4` | ⬜ | `-b`, `-F`, `-E`, `-I`, `-J`, `-L`, `-m`, `-n`, `-q` | |
| `mkswap` | ⬜ | `-f`, `-p`, `-U`, `-L` | |
| `swapon` | ⬜ | `-a`, `-d`, `-e`, `-f`, `-p`, `-s`, `-U`, `-v` | JSON: `--json` |
| `swapoff` | ⬜ | `-a`, `-v` | |
| `tune2fs` | ⬜ | `-l`, `-L`, `-U`, `-c`, `-i`, `-j`, `-m`, `-o`, `-r` | JSON: `-l` |
| `resize2fs` | ⬜ | `-f`, `-F`, `-M`, `-p`, `-P`, `-s` | |
| `mountpoint` | ⬜ | `-q`, `-d`, `-x` | JSON: `--json` |
| `losetup` | ⬜ | `-a`, `-d`, `-f`, `-j`, `-n`, `-o`, `-P`, `--show`, `-v` | JSON: `-J` |
| `cryptsetup` | ⬜ | `open`, `close`, `resize`, `status`, `luksFormat`, `luksAddKey`, `luksRemoveKey` | JSON: `--json` |

---

## 10. System Control

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `reboot` | ⬜ | `-f`, `-p`, `-w` | |
| `shutdown` | ⬜ | `-h`, `-P`, `-H`, `-r`, `-k`, `-c`, `-t`, `-a`, `--no-wall` | |
| `poweroff` | ⬜ | `-f`, `-w`, `-p` | |
| `halt` | ⬜ | `-f`, `-p`, `-w` | |
| `wall` | ⬜ | `-n`, `-t`, `-g`, `-G` | |
| `write` | ⬜ | | |
| `chvt` | ⬜ | | |
| `tty` | ⬜ | `-s`, `--silent` | |
| `clear` | ⬜ | | |
| `reset` | ⬜ | `-I`, `-q`, `-w`, `-e` | |
| `setterm` | ⬜ | `-term`, `-reset`, `-initialize`, `-cursor`, `-blank`, `-powersave`, `-foreground`, `-background`, `-ulink`, `-store` | |

---

## 11. Hardware Info

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `lspci` | ⬜ | `-n`, `-v`, `-b`, `-d`, `-s`, `-i`, `-m`, `-x`, `-D`, `-t` | JSON: `-v -nn` (parsable) |
| `lsusb` | ⬜ | `-v`, `-d`, `-t`, `-s`, `-p` | |
| `lsipc` | ⬜ | `-i`, `-g`, `-m`, `-q`, `-s`, `-u` | JSON: `--json` |
| `lslocks` | ⬜ | `-u`, `-n`, `-p` | JSON: `--json` |
| `lsmem` | ⬜ | `-a`, `-o`, `-J`, `-p`, `-x` | JSON: `-J` |
| `lsmod` | ⬜ | | JSON: `--json` |
| `modprobe` | ⬜ | `-a`, `-r`, `-n`, `-v`, `-C`, `-d`, `-D`, `-c`, `-S` | |
| `modinfo` | ⬜ | `-a`, `-d`, `-F`, `-k`, `-n`, `-p`, `-V` | JSON: `-F json` |
| `setpci` | ⬜ | `-v`, `-D`, `-s`, `-d`, `-y` | |
| `setcap` | ⬜ | `-v`, `-r`, `--drop` | |
| `getcap` | ⬜ | `-v` | |
| `capsh` | ⬜ | `--print`, `--decode`, `--supports`, `--uid`, `--gid`, `--keep`, `--drop`, `--add`, `--inh` | JSON: `--print=capable_json` |
| `udevadm` | ⬜ | `info`, `trigger`, `settle`, `control`, `monitor`, `test-builtin`, `test` | JSON: `info -q all` |
| `hwinfo` | ⬜ | `-all`, `-short`, `-block`, `-cpu`, `-disk`, `-gfxcard`, `-network`, `-pci`, `-usb` | JSON: `--all --json` |

---

## 12. Logging/Debug

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `dmesg` | ⬜ | `-c`, `-C`, `-T`, `-D`, `-E`, `-f`, `-H`, `-k`, `-l`, `-n`, `-r`, `-s`, `-t`, `-w`, `-x`, `-P`, `--level`, `--facility`, `--human`, `--raw` | JSON: `--json` |
| `journalctl` | ⬜ | `-b`, `-f`, `-k`, `-u`, `-r`, `-n`, `-o`, `-a`, `-c`, `--since`, `--until`, `--follow`, `--rotate`, `-vacuum`, `-S`, `-D`, `--user`, `--disk-usage` | JSON: `-o json` |
| `logger` | ⬜ | `-i`, `-f`, `-p`, `-s`, `-t`, `-u`, `--journald`, `--id` | |
| `strace` | ⬜ | `-p`, `-f`, `-ff`, `-o`, `-e`, `-s`, `-v`, `-yy`, `-E`, `-c`, `-C` | JSON: `-f -o strace.log` (parseable) |
| `ltrace` | ⬜ | `-c`, `-C`, `-d`, `-e`, `-f`, `-l`, `-n`, `-o`, `-p`, `-s`, `-S`, `-t`, `-x` | |
| `perf` | ⬜ | `record`, `report`, `stat`, `top`, `list`, `script`, `trace`, `kmem`, `lock` | JSON: `report --stdio` (json events) |

---

## 13. Systemd (systemd replacements)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `systemctl` | ⬜ | `start`, `stop`, `restart`, `reload`, `status`, `enable`, `disable`, `is-enabled`, `is-active`, `list-units`, `list-unit-files`, `daemon-reload`, `show`, `cat`, `set-property`, `reset-failed`, `list-jobs`, `list-timers`, `list-sockets` | JSON: `--output=json` |
| `systemd-analyze` | ⬜ | `time`, `blame`, `critical-chain`, `dot`, `dump`, `set-log-level`, `get-log-level`, `verify`, `calendar`, `unit-files`, `unit-paths`, `exit-statuses` | JSON: `--json=` |
| `systemd-cgtop` | ⬜ | `-p`, `-k`, `-K`, `-t`, `-m`, `--batch` | |
| `systemd-resolve` | ⬜ | `status`, `query`, `service`, `openpgp`, `statistics`, `reset-statistics`, `flush-caches` | JSON: `--json=` |
| `systemd-mount` | ⬜ | | |
| `systemd-escape` | ⬜ | `-m`, `-s`, `-p` | |
| `loginctl` | ⬜ | `list-sessions`, `list-users`, `list-seats`, `show-session`, `show-user`, `show-seat`, `activate`, `lock-session`, `unlock-session`, `terminate-session` | JSON: `--output=json` |
| `timedatectl` | ⬜ | `status`, `set-time`, `set-timezone`, `list-timezones`, `set-ntp`, `set-local-rtc` | JSON: `--json=` |
| `localectl` | ⬜ | `status`, `set-locale`, `list-locales`, `set-keymap`, `list-keymaps`, `set-x11-keymap`, `list-x11-keymap-models` | JSON: `--json=` |
| `hwclock` | ⬜ | `-r`, `-w`, `-s`, `--show`, `--hctosys`, `--systohc`, `--adjust`, `--getepoch`, `--setepoch`, `--noadjfile`, `--adjfile` | JSON: `--json` |

---

## 14. Archives

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `tar` | ⬜ | `-c`, `-x`, `-t`, `-v`, `-f`, `-z`, `-j`, `-J`, `-C`, `-p`, `-P`, `--exclude`, `--files-from`, `--transform`, `--verify`, `-W`, `-M` | JSON: `--list --format=json` |
| `gzip` | ⬜ | `-d`, `-k`, `-l`, `-v`, `-r`, `-S`, `-t`, `-1` to `-9`, `--best`, `--fast` | JSON: `-l` |
| `gunzip` | ⬜ | `-c`, `-f`, `-k`, `-l`, `-n`, `-N`, `-r`, `-t`, `-v` | |
| `zcat` | ⬜ | `-f`, `-h`, `-l`, `-L`, `-n`, `-N`, `-q`, `-r`, `-S`, `-v` | |
| `zgrep` | ⬜ | `-e`, `-f`, `-h`, `-i`, `-l`, `-n`, `-v`, `-x` | |
| `bzip2` | ⬜ | `-d`, `-z`, `-k`, `-f`, `-v`, `-1` to `-9` | JSON: `--keep` |
| `bunzip2` | ⬜ | `-f`, `-k`, `-v` | |
| `xz` | ⬜ | `-d`, `-z`, `-k`, `-f`, `-v`, `-0` to `-9`, `-e`, `--fast`, `--best` | JSON: `--list` |
| `unxz` | ⬜ | `-k`, `-f`, `-v` | |
| `zip` | ⬜ | `-r`, `-d`, `-u`, `-f`, `-F`, `-q`, `-9`, `-0`, `-e`, `-b`, `-x` | |
| `unzip` | ⬜ | `-l`, `-d`, `-o`, `-j`, `-v`, `-t`, `-x`, `-z` | JSON: `-l` |
| `7z` | ⬜ | `a`, `x`, `l`, `t`, `d`, `u`, `-p`, `-m`, `-mx`, `-m0`, `-m1` | JSON: `l -slt` |
| `7za` | ⬜ | | |
| `ar` | ⬜ | `-d`, `-m`, `-p`, `-q`, `-r`, `-t`, `-x`, `-c`, `-s` | |

---

## 15. Build Tools

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `make` | ⬜ | `-f`, `-n`, `-j`, `-k`, `-B`, `-C`, `-o`, `-p`, `-s`, `-w` | JSON: `--print-data-base` |
| `cmake` | ⬜ | `-B`, `-D`, `-G`, `-S`, `--build`, `--install`, `--target`, `--fresh` | JSON: `--trace` |
| `meson` | ⬜ | `setup`, `configure`, `compile`, `install`, `test`, `clean`, `introspect` | JSON: `introspect` |
| `ninja` | ⬜ | `-C`, `-f`, `-j`, `-k`, `-n`, `-t`, `-v`, `-d`, `--query` | JSON: `--query` |
| `gcc` | ⬜ | `-o`, `-c`, `-E`, `-S`, `-Wall`, `-Wextra`, `-O`, `-g`, `-I`, `-L`, `-l`, `-D`, `-U`, `-std`, `-m`, `-f` | |
| `g++` | ⬜ | | |
| `ld` | ⬜ | `-o`, `-L`, `-l`, `-T`, `-shared`, `-static`, `-r` | |
| `objdump` | ⬜ | `-d`, `-D`, `-S`, `-s`, `-t`, `-x`, `-h`, `-p`, `-r`, `-R`, `-g`, `-G`, `-j`, `-M` | |
| `objcopy` | ⬜ | `-I`, `-O`, `-F`, `-B`, `-R`, `--remove-section`, `--add-section`, `--only-section` | |
| `nm` | ⬜ | `-a`, `-g`, `-u`, `-D`, `-B`, `-S`, `-s`, `-r`, `-t`, `-f`, `--size-sort`, `--extern-only` | JSON: `--format=just-symbols` |
| `strings` | ⬜ | (see Text Processing) | |
| `strip` | ⬜ | `-s`, `-d`, `-g`, `-S`, `-D`, `-K`, `-N`, `-o`, `-p`, `-x`, `-X` | |
| `pkg-config` | ⬜ | `--cflags`, `--libs`, `--static`, `--modversion`, `--variable`, `--exists`, `--uninstalled`, `--print-errors`, `--short-errors`, `--silence-errors`, `--print-requires`, `--print-requires-private`, `--max-version` | JSON: `--print-requires --print-requires-private` |
| `autoconf` | ⬜ | | |
| `automake` | ⬜ | | |
| `libtool` | ⬜ | | |

---

## 16. Permissions

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `chmod` | ⬜ | `-c`, `-f`, `-v`, `-R`, `--reference`, `--changes`, `--no-preserve-root`, `--preserve-root` | JSON: `--changes` |
| `chown` | ⬜ | `-c`, `-f`, `-h`, `-R`, `-v`, `--from`, `--reference`, `--changes` | JSON: `--changes` |
| `chgrp` | ⬜ | `-c`, `-f`, `-h`, `-R`, `-v`, `--reference`, `--changes` | JSON: `--changes` |
| `getfacl` | ⬜ | `-a`, `-d`, `-R`, `-p`, `-n`, `--access`, `--default`, `--no-effective`, `--skip-base`, `--all-effective`, `--set`, `--set-file`, `--mask` | JSON: `--no-effective` |
| `setfacl` | ⬜ | `-R`, `-b`, `-k`, `-n`, `-d`, `-m`, `-M`, `-x`, `-X`, `--restore`, `--set`, `--set-file`, `--mask` | |
| `umask` | ⬜ | `-S`, `-p` | |

---

## 17. Shells

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `bash` | ⬜ | `-c`, `-s`, `-r`, `-i`, `-l`, `-n`, `-x`, `-v`, `--noprofile`, `--norc`, `--rcfile`, `--init-file`, `--dump-strings`, `--dump-po-filenames` | |
| `sh` | ⬜ | | |
| `dash` | ⬜ | | |
| `zsh` | ⬜ | | |
| `env` | ⬜ | `-i`, `-u`, `--ignore-environment`, `--null`, `-0` | JSON: `-0` |
| `printenv` | ⬜ | `-0`, `--null` | JSON: `-0` |
| `set` | ⬜ | `-a`, `-b`, `-e`, `-u`, `-x`, `-o`, `-h` | |
| `unset` | ⬜ | `-f`, `-v` | |
| `export` | ⬜ | `-n`, `-p`, `-f` | |
| `source` | ⬜ | | |
| `alias` | ⬜ | `-p` | |
| `unalias` | ⬜ | `-a` | |
| `history` | ⬜ | `-c`, `-d`, `-a`, `-n`, `-r`, `-w`, `-p`, `-s` | JSON: `-w -n` |
| `fc` | ⬜ | `-e`, `-l`, `-n`, `-r`, `-s` | |
| `bind` | ⬜ | `-q`, `-l`, `-p`, `-P`, `-m`, `-s`, `-u`, `-V`, `-v` | |
| `complete` | ⬜ | `-p`, `-r`, `-D`, `-E`, `-F`, `-G`, `-W`, `-P`, `-S`, `-A`, `-G`, `-W`, `-a`, `-b`, `-c`, `-d`, `-e`, `-f`, `-j`, `-k`, `-n`, `-o`, `-s`, `-u`, `-v`, `-X`, `-M`, `-o`, `-T` | |
| `compgen` | ⬜ | `-a`, `-b`, `-c`, `-d`, `-e`, `-f`, `-g`, `-j`, `-k`, `-u`, `-v`, `-w`, `-A`, `-G`, `-W`, `-F`, `-P`, `-S`, `-X` | |

---

## Legend

- ⬜ Not implemented
- ⏳ In progress
- ✅ Implemented
- 🔄 Needs refactoring
- ⚠️ Partially implemented

## Output Format Standards

1. **Default**: JSON output for all listing/status commands
2. **Flags**: `--format=table` or `--human-readable` for tabular output
3. **Stable**: Field names must not change between versions
4. **Null handling**: Explicit `null` vs empty strings
5. **Timestamps**: ISO 8601 format (RFC 3339)
6. **Sizes**: Bytes (int) by default, human-readable as option
7. **Versions**: Semantic versioning for all tools
8. **Exit codes**: Standard POSIX exit codes with JSON error output to stderr

---

## Priority Implementation Order

1. **Phase 1 - Core Package Management**: `app`, `app-get`, `rpg`, `rpg-query`
2. **Phase 2 - Basic System Info**: `uname`, `lsb_release`, `hostname`, `free`, `df`, `lsblk`
3. **Phase 3 - File Operations**: `ls`, `cp`, `mv`, `rm`, `mkdir`, `cat`
4. **Phase 4 - Network**: `ip`, `ss`, `ping`, `curl`, `ssh`
5. **Phase 5 - Process**: `ps`, `top`, `kill`, `killall`
6. **Phase 6 - User Management**: `useradd`, `userdel`, `usermod`, `passwd`
7. **Phase 7 - System Control**: `systemctl` replacement, `reboot`, `shutdown`
