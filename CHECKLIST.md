# Rustica OS Command Implementation Checklist

> **Output Format Requirement**: All commands must output stable, machine-readable format by default (JSON/CSV). Human-readable tables available via `--format=table` or `--human-readable` flag.

---

## Naming Rules

Every command keeps its familiar name (`ls`, `grep`, `ip`, `systemctl`, `tar`, ...) but the package tools: Debian's two layers (dpkg under apt) and their families of commands are replaced by **one tool, `rpg`**, with subcommands (decided 2026-11-16; the plan is `PACKAGES.md` in the Rustux project, phase E). There is no `app`, `app-get`, `app-cache`, `app-key`, `applicate`, `rpg-deb`, `rpg-query`, ...

| Ubuntu | Rustica |
|--------|---------|
| `apt`, `apt-get`, `apt-cache`, `apt-mark`, `apt-key`, `add-apt-repository`, `aptitude` | `rpg` and its subcommands (below) |
| `dpkg`, `dpkg-deb`, `dpkg-query` | `rpg` and its subcommands (below) |

Package files are `.rpg`. One repository for every edition: `https://rustux.com/download/rustica` (suite `redfox`, section `main`); retired releases move to `https://archive.rustux.com/download/rustica`. Indexes are signed with Ed25519 (the archive key `rustux-archive-2026`).

---

## 1. Package Management (`rpg`)

### 1.1 Repositories and updates (apt's part)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `rpg update` | ⬜ | | The sources' `Release` (signature checked) and indexes; apt update |
| `rpg upgrade` | ⬜ | `-y`, `--download-only` | Every upgradable package, new dependencies included; apt upgrade and full-upgrade |
| `rpg install NAME...` | ⬜ | `-y`, `--download-only`, `--no-recommends` | With dependencies; apt install |
| `rpg install ./FILE.rpg` | ⬜ | `-y` | A package file; dpkg -i |
| `rpg remove NAME...` | ⬜ | `-y`, `--purge` | apt remove / purge |
| `rpg autoremove` | ⬜ | `-y` | Dependencies no longer needed |
| `rpg search TEXT` | ⬜ | `--names-only`, `--installed` | apt search. JSON: `--json` |
| `rpg show NAME` | ⬜ | | apt show. JSON: `--json` |
| `rpg policy NAME` | ⬜ | | Installed and candidate versions, from which source; apt-cache policy |
| `rpg list` | ⬜ | `--installed`, `--upgradable`, `--held` | apt list. JSON: `--json` |
| `rpg depends NAME`, `rpg rdepends NAME` | ⬜ | | apt-cache depends / rdepends |
| `rpg hold NAME`, `rpg unhold NAME` | ⬜ | | apt-mark hold |
| `rpg source list`, `rpg source add LINE`, `rpg source remove N` | ⬜ | | `/etc/rpg/sources.list`; add-apt-repository |
| `rpg key list`, `rpg key add FILE`, `rpg key remove ID` | ⬜ | | `/etc/rpg/keys/`; apt-key |
| `rpg download NAME` | ⬜ | | Into the current folder; apt download |
| `rpg clean` | ⬜ | | `/var/cache/rpg/archives/`; apt clean |

### 1.2 Installed packages (dpkg's part)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `rpg files NAME` | ⬜ | | The files a package installed; dpkg -L |
| `rpg owner PATH` | ⬜ | | Which package a file belongs to; dpkg -S |
| `rpg verify [NAME]` | ⬜ | | Files changed or missing (SHA-256); dpkg -V |
| `rpg fix` | ⬜ | | Finishes or undoes an interrupted install; dpkg --configure -a, apt --fix-broken |
| `rpg info ./FILE.rpg` | ⬜ | | A package file's control and files; dpkg-deb -I / -c |

### 1.3 Making packages and repositories (the publishing side)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `rpg build DIR` | ⬜ | `-o FILE` | A `.rpg` from a `control` file and a tree; dpkg-deb --build |
| `rpg index DIR` | ⬜ | | `Packages.gz` and `Release` for a folder of packages; apt-ftparchive. Signing is separate: `openssl pkeyutl -sign -rawin` with the archive key |

---

## 2. System Information

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `uname` | ⚠️ | `-a`, `-s`, `-r`, `-v`, `-m`, `-p`, `-i`, `-o` | JSON: `--json`. Rustux: uutils coreutils; GNU text output, no JSON yet; `-o` prints Rustux |
| `uname -a` | ✅ | | All info. Rustux: uutils coreutils |
| `lsb_release` | ⬜ | `-a`, `-i`, `-d`, `-r`, `-c` | JSON: `--json` |
| `hostname` | ⚠️ | `-i`, `-I`, `-s`, `-f`, `-d`, `-y` | JSON: `--json`. Rustux: C program; shows or sets `/etc/hostname`; no flags yet |
| `hostnamectl` | ⬜ | `status`, `set-hostname`, `set-chassis` | JSON: `--json=pretty` |
| `uptime` | ⚠️ | `-p`, `--since`, `-s` | JSON: `--json`. Rustux: C program; `--json` |
| `free` | ✅ | `-h`, `-b`, `-k`, `-m`, `-g`, `--si`, `-t`, `-o`, `-l` | JSON: `--json`. Rustux: C program; `-k`, `-m`, `-g`, `-h`, `--json` |
| `vmstat` | ⬜ | `-s`, `-d`, `-D`, `-p`, `-S`, `-m`, `-a`, `-t`, `-w` | JSON: `--json` |
| `iostat` | ⬜ | `-c`, `-d`, `-x`, `-k`, `-m`, `-p`, `-t`, `-z`, `-h` | JSON: `--json` |
| `lsblk` | ⚠️ | `-a`, `-b`, `-f`, `-m`, `-d`, `-o`, `-O`, `-P`, `-J`, `-p` | JSON: `-J` / `--json`. Rustux: C program asking devmgr; tree of disks and partitions (step 13); `-b`, `-n`, `-l`, `-d`, `-f`, `-o`, `-J`; no `-a`, `-m`, `-O`, `-P`, `-p` |
| `lsblk -f` | ✅ | | Filesystems. Rustux: FSTYPE, LABEL, UUID by signature (vfat, exfat, ntfs, BitLocker, ext2/3/4, swap, iso9660) |
| `lsblk -m` | ⬜ | | Permissions |
| `blkid` | ⚠️ | `-o`, `-p`, `-s`, `-t`, `-u`, `-c`, `-L`, `-U` | JSON: `-o export` / `--json`. Rustux: C program (step 13); `-o full/value/export`, `-s`; device names are devmgr's (`vda1`), not `/dev` paths; no `-p`, `-t`, `-L`, `-U` |
| `df` | ⚠️ | `-h`, `-H`, `-i`, `-k`, `-l`, `-P`, `-T`, `-x`, `-t`, `-a` | JSON: `--output=json`. Rustux: uutils coreutils; GNU text output, no JSON yet; reads `/etc/mtab` (tmpfs, bootfs, devfs, and mounted FAT disks with their cluster counts, step 14) |
| `du` | ⚠️ | `-h`, `-s`, `-c`, `-a`, `-x`, `-d`, `--max-depth`, `--time` | JSON: `--json`. Rustux: uutils coreutils; GNU text output, no JSON yet |
| `mount` | ⚠️ | `-t`, `-a`, `-o`, `-v`, `-f`, `-l`, `-n`, `--bind`, `--move` | JSON: `--json`. Rustux: C program (step 14); lists `/etc/mtab`, `-t vfat`, `-o ro/rw/noauto/nofail`, `-a` from `/etc/fstab` (run at boot: EFI partition at `/boot/efi`); sources: names, `LABEL=`, `UUID=`, `PARTLABEL=`, `EFI`; vfat, ext2 (step 15; `--root` for init's root filesystem), ext3 and ext4 (step 16: journal replayed at mount); no `-v`, `-f`, `-l`, `-n`, `--bind`, `--move`, JSON |
| `umount` | ⚠️ | `-f`, `-l`, `-n`, `-r`, `-d`, `-v`, `-a` | Rustux: C program (step 14); `-f`, `-a`; by directory or device; no `-l`, `-n`, `-r`, `-d`, `-v` |
| `findmnt` | ⬜ | `-a`, `-s`, `-t`, `-o`, `-J`, `-p`, `-r`, `-u`, `-U` | JSON: `-J` |
| `stat` | ⚠️ | `-c`, `-f`, `-L`, `-t`, `-Z` | JSON: `-c %` format. Rustux: uutils coreutils; GNU text output, no JSON yet |
| `watch` | ⬜ | `-n`, `-d`, `-h`, `-t`, `-g`, `-e`, `-x` | |

---

## 3. File Operations

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `ls` | ⚠️ | `-l`, `-a`, `-h`, `-R`, `-t`, `-r`, `-S`, `-i`, `-F`, `-p` | JSON: `--json`. Rustux: uutils coreutils; GNU text output, no JSON yet; `ls -l` shows modes, sizes and UTC times |
| `ls -l` | ⚠️ | | Long format (JSON default). Rustux: text output (no JSON yet) |
| `ls -la` | ✅ | | All, long. Rustux: uutils coreutils |
| `tree` | ⬜ | `-a`, `-d`, `-f`, `-F`, `-h`, `-L`, `-p`, `-s`, `-u`, `-g` | JSON: `-J` |
| `pwd` | ⚠️ | `-L`, `-P` | JSON: `--json`. Rustux: uutils coreutils; GNU text output, no JSON yet |
| `cd` | ✅ | | Shell built-in. Rustux: shell built-in |
| `cp` | ⚠️ | `-r`, `-a`, `-v`, `-p`, `-n`, `-i`, `-b`, `--backup` | JSON: `--json` (log only). Rustux: uutils coreutils; GNU text output, no JSON yet |
| `mv` | ⚠️ | `-i`, `-f`, `-n`, `-v`, `-b` | JSON: `--json` (log only). Rustux: uutils coreutils; GNU text output, no JSON yet |
| `rm` | ⚠️ | `-r`, `-f`, `-i`, `-v`, `-d` | JSON: `--json` (log only). Rustux: uutils coreutils; GNU text output, no JSON yet |
| `rmdir` | ✅ | `-p`, `-v` | Rustux: uutils coreutils |
| `mkdir` | ⚠️ | `-p`, `-v`, `-m` | JSON: `--json` (log only). Rustux: uutils coreutils; GNU text output, no JSON yet |
| `install` | ⬜ | `-m`, `-o`, `-g`, `-d`, `-v`, `-b` | |
| `file` | ⬜ | `-b`, `-i`, `-z`, `-k`, `-L`, `-s`, `--mime-type` | JSON: `--json` |
| `touch` | ⚠️ | `-a`, `-m`, `-c`, `-d`, `-r`, `-t` | Rustux: uutils coreutils; GNU text output, no JSON yet |
| `basename` | ⚠️ | `-a`, `-s` | JSON: `--json`. Rustux: uutils coreutils; GNU text output, no JSON yet |
| `dirname` | ⚠️ | | JSON: `--json`. Rustux: uutils coreutils; GNU text output, no JSON yet |
| `realpath` | ⚠️ | `-e`, `-m`, `-s`, `-z` | JSON: `--json`. Rustux: uutils coreutils; GNU text output, no JSON yet |
| `readlink` | ⚠️ | `-f`, `-e`, `-m`, `-n`, `-v`, `-z` | JSON: `--json`. Rustux: uutils coreutils; GNU text output, no JSON yet |

---

## 4. Text Processing

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `cat` | ⚠️ | `-n`, `-b`, `-s`, `-E`, `-T`, `-v`, `-e`, `-A` | Rustux: uutils coreutils; GNU text output, no JSON yet |
| `less` | ⚠️ | `-N`, `-S`, `-F`, `-R`, `-m`, `-M`, `-i`, `--follow-name` | Rustux: `tools`; search, `-N`, `-S`, `-F`, `-i`; no `--follow-name` |
| `more` | ⚠️ | `-d`, `-l`, `-f`, `-p`, `-c`, `-s`, `-u` | Rustux: link to `less` |
| `head` | ⚠️ | `-n`, `-c`, `-v`, `-q`, `-z` | JSON: `--json` (line counts). Rustux: uutils coreutils; GNU text output, no JSON yet |
| `tail` | ⚠️ | `-n`, `-f`, `-F`, `--pid`, `-v`, `-q`, `-z`, `--retry` | JSON: `--json` (follow mode). Rustux: uutils coreutils; GNU text output, no JSON yet; `-f` polls once a second (no inotify) |
| `nl` | ⚠️ | `-b`, `-h`, `-i`, `-l`, `-n`, `-p`, `-s`, `-v`, `-w` | JSON: `--json`. Rustux: uutils coreutils; GNU text output, no JSON yet |
| `wc` | ⚠️ | `-c`, `-l`, `-w`, `-m`, `-L` | JSON: `--json`. Rustux: uutils coreutils; GNU text output, no JSON yet |
| `sort` | ⚠️ | `-b`, `-d`, `-f`, `-g`, `-i`, `-M`, `-h`, `-n`, `-R`, `-r` | JSON: `--json`. Rustux: uutils coreutils; GNU text output, no JSON yet |
| `sort -u` | ✅ | | Unique. Rustux: uutils coreutils |
| `sort -k` | ✅ | | Key field. Rustux: uutils coreutils |
| `sort -t` | ✅ | | Field separator. Rustux: uutils coreutils |
| `uniq` | ⚠️ | `-c`, `-d`, `-D`, `-u`, `-i`, `-z`, `-f`, `-s`, `-w` | JSON: `--json`. Rustux: uutils coreutils; GNU text output, no JSON yet |
| `cut` | ⚠️ | `-b`, `-c`, `-d`, `-f`, `-s`, `--complement`, `--output-delimiter` | Rustux: uutils coreutils; GNU text output, no JSON yet |
| `paste` | ⚠️ | `-d`, `-s`, `-z` | Rustux: uutils coreutils; GNU text output, no JSON yet |
| `tr` | ⚠️ | `-c`, `-C`, `-d`, `-s`, `-t` | Rustux: uutils coreutils; GNU text output, no JSON yet |
| `column` | ⬜ | `-t`, `-s`, `-o`, `-x`, `-n` | JSON: `--json` |
| `fold` | ⚠️ | `-b`, `-s`, `-w` | Rustux: uutils coreutils; GNU text output, no JSON yet |
| `fmt` | ⚠️ | `-w`, `-p`, `-s`, `-t`, `-c` | Rustux: uutils coreutils; GNU text output, no JSON yet |
| `join` | ⚠️ | `-a`, `-v`, `-1`, `-2`, `-j`, `-o`, `-t`, `-i` | JSON: `--json`. Rustux: uutils coreutils; GNU text output, no JSON yet |
| `awk` | ⬜ | `-F`, `-v`, `-f` | |
| `sed` | ⬜ | `-n`, `-e`, `-f`, `-E`, `-r`, `-i`, `-z` | |
| `grep` | ✅ | `-i`, `-v`, `-c`, `-l`, `-L`, `-n`, `-H`, `-h`, `-o`, `-q`, `-m`, `-A`, `-B`, `-C`, `-R`, `-r`, `-E`, `-F`, `-e`, `-f`, `-x`, `-w`, `-z`, `--color`, `--exclude`, `--exclude-dir`, `--include`, `--binary-files` | JSON: `--json` (matches only). Rustux: `tools` (Rust regex); `--json` prints matches as JSON lines; BRE, ERE (`-E`), fixed (`-F`); `-P` not supported |
| `egrep` | ✅ | | Extended grep. Rustux: `tools`, same as `grep -E` |
| `fgrep` | ✅ | | Fixed grep. Rustux: `tools`, same as `grep -F` |
| `strings` | ⬜ | `-a`, `-f`, `-n`, `-t`, `-e`, `-o` | JSON: `--json` |

---

## 5. File Finding

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `find` | ⚠️ | `-name`, `-iname`, `-type`, `-size`, `-mtime`, `-atime`, `-ctime`, `-perm`, `-user`, `-group`, `-uid`, `-gid`, `-empty`, `-executable`, `-readable`, `-writable`, `-depth`, `-maxdepth`, `-mindepth`, `-xdev`, `-x`, `-noleaf`, `-regextype`, `-exec`, `-ok`, `-print`, `-print0`, `-ls`, `-delete`, `-prune` | JSON: `--json`. Rustux: uutils findutils (`-user`/`-group` see root only; no JSON yet) |
| `locate` | ⬜ | `-i`, `-c`, `-e`, `-d`, `-r`, `-w`, `-l`, `-0`, `-S` | JSON: `--json` |
| `updatedb` | ⬜ | `-l`, `-U`, `-o`, `-e`, `-f`, `-q` | |
| `which` | ⬜ | `-a`, `-s` | JSON: `--json` |
| `whereis` | ⬜ | `-b`, `-m`, `-s`, `-u` | JSON: `--json` |
| `xargs` | ✅ | `-0`, `-a`, `-t`, `-n`, `-P`, `-p`, `-I`, `-i`, `-L`, `-d` | Rustux: uutils findutils (patched to look up `$PATH` itself); `-P` works |

---

## 6. Networking

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `ip` | ⚠️ | `address`, `route`, `link`, `netns`, `neigh`, `rule` | JSON: `-j` / `-json`. Rustux: C program (step D1, D4); `addr`, `route`, `link`, `-4`/`-6`, `-s`, `-br`; no `netns`, `neigh`, `rule`, no JSON |
| `ip addr` | ✅ | `add`, `del`, `show`, `-a`, `-dev` | JSON: `-j`. Rustux: `add`, `del`, `show [IF]` (root changes); no JSON |
| `ip route` | ⚠️ | `add`, `del`, `show`, `get`, `replace` | JSON: `-j`. Rustux: `show`, `add`/`del default via`; no `get`, `replace`, other routes; no JSON |
| `ip link` | ✅ | `set`, `show`, `up`, `down`, `-name` | JSON: `-j`. Rustux: `show`, `set IF up`/`down`; no JSON |
| `ip neigh` | ⬜ | `show`, `flush`, `add`, `del`, `replace` | JSON: `-j` |
| `ss` | ⚠️ | `-t`, `-u`, `-w`, `-x`, `-a`, `-l`, `-p`, `-n`, `-m`, `-o`, `-i`, `-s`, `-4`, `-6`, `-Z` | JSON: `-j` / `-H`. Rustux: C program (step D2); `-t -u -l -a -n -H`; no `-p`, `-x`, `-m`, `-i`, `-s`, no JSON |
| `ping` | ✅ | `-c`, `-i`, `-W`, `-s`, `-t`, `-4`, `-6`, `-O` | JSON: `-O` (summary). Rustux: C program (step D1, D4); `-c -i -W -w -s -q -4 -6`, names; no `-t`, `-O`, no JSON |
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
| `curl` | ⚠️ | `-o`, `-O`, `-I`, `-v`, `-s`, `-w`, `-H`, `-d`, `-X`, `-A`, `-u`, `-x`, `-k`, `--head`, `--get` | JSON: `-w "%{json}"`. Rustux: Rust, ureq on rustls (step D3); `-o -O -L -I -i -f -s -S -v -k -H -d -X -A -u -m -w --cacert --connect-timeout`; no `-x` (proxies), no `%{json}` |
| `wget` | ⬜ | `-O`, `-o`, `-c`, `-r`, `-b`, `-q`, `-v`, `-t` | |
| `ftp` | ⬜ | | |
| `sftp` | ⬜ | `-b`, `-o`, `-P` | Batch mode |
| `scp` | ⬜ | `-r`, `-p`, `-P`, `-i`, `-v` | |
| `ssh` | ⬜ | `-i`, `-p`, `-L`, `-R`, `-D`, `-v`, `-N`, `-f`, `-n`, `-X` | |
| `ssh-keygen` | ⬜ | `-t`, `-b`, `-C`, `-f`, `-l`, `-p`, `-y`, `-e`, `-q` | JSON: `-l -f` |
| `ssh-copy-id` | ⬜ | `-i`, `-f`, `-o` | |
| `telnet` | ⬜ | | |
| `nc` | ✅ | `-l`, `-p`, `-v`, `-z`, `-w`, `-q`, `-u` | Rustux: C program (step D2), OpenBSD netcat's `-l -k -u -v -z -N -w -s -p`; no `-q` |
| `whois` | ⬜ | `-h`, `-p` | |
| `dig` | ⬜ | `@server`, `+short`, `+json`, `-x`, `-t`, `any`, `ns`, `soa` | JSON: `+json` |
| `nslookup` | ⚠️ | `-type`, `-port`, `-debug` | JSON: `-json`. Rustux: C program (step D3), with `host`; no interactive mode, no JSON |
| `host` | ⚠️ | `-t`, `-v`, a server | Rustux: C program (step D3); A, AAAA, MX, PTR; no JSON |
| `sshd` | ⚠️ | `-d`, `/etc/ssh/sshd_config` | Rustux: Rust, on the sunset crate (step D5); passwords and Ed25519 keys, pseudo-terminals; no scp/sftp subsystems, no forwarding |
| `fwctl` | ✅ | `allow`, `deny`, `append`, `delete`, `default`, `enable`, `disable`, `reload`, `save` | Rustux's own firewall tool (step D5), no iptables/nftables |
| `netcfg` | ✅ | `IFACE dhcp`, `static`, `off`, `dns`, `search`, `apply` | Rustux's own (step D6): `/etc/network.conf` applied at once; Settings > Network uses it |

---

## 7. User Management

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `login` | ⚠️ | `-f`, `-h`, `-p` | Rustux: C program (step 17); init runs it on the console; `-f USER` (autologin); no `-h`, `-p` |
| `logout` | ⬜ | | |
| `whoami` | ⚠️ | | JSON: `--json`. Rustux: uutils coreutils (real users since step 17); GNU text output, no JSON yet |
| `id` | ⚠️ | `-u`, `-g`, `-G`, `-n`, `-r`, `-z` | JSON: `--json`. Rustux: uutils coreutils (the kernel's credentials since step 17); GNU text output, no JSON yet |
| `groups` | ⚠️ | | JSON: `--json`. Rustux: uutils coreutils (step 17); no JSON yet |
| `passwd` | ⚠️ | `-d`, `-e`, `-i`, `-k`, `-l`, `-n`, `-S`, `-u`, `-w` | JSON: `-S`. Rustux: C program (step 17), through authd; `passwd [USER]` only (lock with `usermod -L`) |
| `chsh` | ⬜ | `-s`, `-l` | |
| `su` | ⚠️ | `-`, `-c`, `-s`, `-l`, `-m` | Rustux: C program (step 17), through authd; `-`, `-l`, `-c`; no `-s`, `-m` |
| `sudo` | ⚠️ | `-i`, `-u`, `-s`, `-H`, `-E`, `-v` | JSON: `--json` (validate). Rustux: C program (step 17), through authd; `/etc/sudoers` (users, `%group`, RUNAS, `NOPASSWD:`); `-u`, `-i`, `-s`, `-S`, `-l`; no `-H`, `-E`, `-v`, `-k` |
| `newgrp` | ⬜ | | |
| `useradd` | ⚠️ | `-m`, `-s`, `-d`, `-g`, `-G`, `-u`, `-k`, `-b` | JSON: `--json`. Rustux: C program (step 17); `-m -s -d -g -G -u -c` (skel: `/etc/skel`); no `-k`, `-b`, JSON |
| `userdel` | ⚠️ | `-r`, `-f` | Rustux: C program (step 17); `-r`; no `-f` |
| `usermod` | ⚠️ | `-a`, `-G`, `-l`, `-s`, `-d`, `-m`, `-u`, `-L`, `-U` | Rustux: C program (step 17); `-a -G -g -s -d -c -L -U`; no `-l`, `-m`, `-u` |
| `groupadd` | ⚠️ | `-g`, `-o`, `-r`, `-f` | JSON: `--json`. Rustux: C program (step 17); `-g`; no `-o`, `-r`, `-f` |
| `groupdel` | ✅ | | Rustux: C program (step 17); not a user's primary group |
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
| `ps` | ⚠️ | `-a`, `-u`, `-x`, `-e`, `-f`, `-l`, `-y`, `-o`, `--sort`, `-p`, `--ppid`, `-t`, `-C` | JSON: `--format=json`. Rustux: C program using kernel object info; `--json`; no BSD/UNIX flags yet |
| `ps aux` | ⬜ | | BSD style |
| `ps -ef` | ⬜ | | UNIX style |
| `top` | ⚠️ | `-b`, `-n`, `-d`, `-p`, `-u`, `-U`, `-H` | JSON batch: `-b -n 1`. Rustux: C program; `-b`, `-n`, `-d`; no JSON yet |
| `htop` | ⬜ | | TUI only |
| `atop` | ⬜ | `-b`, `-w`, `-M`, `-P`, `-C`, `-m`, `-A`, `-R` | JSON: `--json` |
| `nice` | ⬜ | `-n`, `--adjustment` | |
| `renice` | ⬜ | `-n`, `-p`, `-u`, `-g` | JSON: `--json` |
| `kill` | ⚠️ | `-l`, `-s`, `-SIGNAL` | JSON: `-l`. Rustux: C program; ends the process (no signals yet, `-SIG` ignored) |
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
| `mount` | ⚠️ | (see System Info) | |
| `umount` | ⚠️ | (see System Info) | |
| `lsblk` | ⚠️ | (see System Info) | |
| `blkid` | ⚠️ | (see System Info) | |
| `fdisk` | ⚠️ | `-l` | Rustux: listing only (step 13); tables are changed with `gpt` |
| `gpt` | ✅ | `show`, `create`, `add`, `remove`, `label`, `type`, `destroy`, `types` | Rustux's GPT tool (step 13), FreeBSD-like |
| `blk` | ✅ | `read`, `write`, `info`, `rescan`, `flush` | Rustux's: block devices from the shell while they are not in `/dev` (step 13) |
| `findmnt` | ⬜ | (see System Info) | |
| `df` | ⚠️ | (see System Info) | Rustux: uutils coreutils; GNU text output, no JSON yet; reads `/etc/mtab` (tmpfs, bootfs, devfs, FAT disks) |
| `du` | ⚠️ | (see System Info) | Rustux: uutils coreutils; GNU text output, no JSON yet |
| `fsck` | ⬜ | `-a`, `-A`, `-C`, `-f`, `-M`, `-N`, `-P`, `-r`, `-R`, `-T`, `-V`, `-y` | JSON: `--json` |
| `fsck.ext4` | ⚠️ | `-b`, `-c`, `-f`, `-p`, `-y`, `-n`, `-v` | Rustux: `fsck.ext2` under this name (step 16); replays the journal (`-a`/`-p`/`-y`), then checks (passes, extents, checksums); `-a` corrects the free counts; no `-b`, `-c`, other repairs |
| `mkfs` | ⬜ | `-t`, `-c`, `-v`, `-n` | Rustux: `mkfs.fat`, `mkfs.ext2` and `mkfs.ext4` instead (below) |
| `mkfs.ext2` / `mke2fs` | ⚠️ | `-b`, `-i`, `-L`, `-U`, `-m`, `-I` | Rustux: C program (step 15); ext2 as Linux's mke2fs makes it (without the resize inode); no `-t`, `-O`, `-N`, `-d` |
| `fsck.ext2` / `fsck.ext3` | ⚠️ | `-a`, `-n`, `-v` | Rustux: C program (steps 15, 16); checks ext2/3/4 (e2fsck's five passes, extents, checksums), replays the journal; `-a` corrects the free counts |
| `mkfs.fat` / `mkfs.vfat` | ✅ | `-F`, `-n`, `-i`, `-s`, `-I` | Rustux: C program (step 14); FAT12/16/32, type and cluster size by size as Windows formats |
| `fsck.fat` / `fsck.vfat` | ⚠️ | `-a`, `-n`, `-v` | Rustux: C program (step 14); checks only; `-a` corrects the FSInfo free count |
| `mkfs.ext4` | ⚠️ | `-b`, `-F`, `-E`, `-I`, `-J`, `-L`, `-m`, `-n`, `-q` | Rustux: C program (step 16); Linux's ext4 defaults (without the resize inode), `-b`, `-i`, `-J size=`, `-L`, `-U`, `-m`, `-O ^has_journal`, `-I` (whole disk), `-F`/`-q` accepted; no `-E`, `-n` |
| `mkswap` | ⬜ | `-f`, `-p`, `-U`, `-L` | |
| `swapon` | ⬜ | `-a`, `-d`, `-e`, `-f`, `-p`, `-s`, `-U`, `-v` | JSON: `--json` |
| `swapoff` | ⬜ | `-a`, `-v` | |
| `tune2fs` | ⚠️ | `-l`, `-L`, `-U`, `-c`, `-i`, `-j`, `-m`, `-o`, `-r` | JSON: `-l`. Rustux: C program (step 16); `-l` as Linux's, `-L`; no other changes yet |
| `resize2fs` | ⬜ | `-f`, `-F`, `-M`, `-p`, `-P`, `-s` | |
| `mountpoint` | ⬜ | `-q`, `-d`, `-x` | JSON: `--json` |
| `losetup` | ⬜ | `-a`, `-d`, `-f`, `-j`, `-n`, `-o`, `-P`, `--show`, `-v` | JSON: `-J` |
| `cryptsetup` | ⬜ | `open`, `close`, `resize`, `status`, `luksFormat`, `luksAddKey`, `luksRemoveKey` | JSON: `--json` |

---

## 10. System Control

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `reboot` | ⚠️ | `-f`, `-p`, `-w` | Rustux: C program (ACPI reset); root (or sudo): init ends the session and writes the disks out first (step 17); no flags yet |
| `shutdown` | ⬜ | `-h`, `-P`, `-H`, `-r`, `-k`, `-c`, `-t`, `-a`, `--no-wall` | |
| `poweroff` | ⚠️ | `-f`, `-w`, `-p` | Rustux: C program (ACPI S5); root (or sudo): init ends the session and writes the disks out first (step 17); no flags yet |
| `halt` | ⬜ | `-f`, `-p`, `-w` | |
| `wall` | ⬜ | `-n`, `-t`, `-g`, `-G` | |
| `write` | ⬜ | | |
| `chvt` | ⬜ | | |
| `tty` | ⬜ | `-s`, `--silent` | |
| `clear` | ✅ | Rustux: shell built-in |
| `reset` | ⬜ | `-I`, `-q`, `-w`, `-e` | |
| `setterm` | ⬜ | `-term`, `-reset`, `-initialize`, `-cursor`, `-blank`, `-powersave`, `-foreground`, `-background`, `-ulink`, `-store` | |

---

## 11. Hardware Info

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `lspci` | ⚠️ | `-n`, `-v`, `-b`, `-d`, `-s`, `-i`, `-m`, `-x`, `-D`, `-t` | JSON: `-v -nn` (parsable). Rustux: C program (step 12) asking devmgr; `-n`, `-nn`, `-v`, `-k`, `-D`, `-s`, `-d`, `--json`; no `-b`, `-i`, `-m`, `-x`, `-t` |
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
| `dmesg` | ⚠️ | `-c`, `-C`, `-T`, `-D`, `-E`, `-f`, `-H`, `-k`, `-l`, `-n`, `-r`, `-s`, `-t`, `-w`, `-x`, `-P`, `--level`, `--facility`, `--human`, `--raw` | JSON: `--json`. Rustux: C program; prints the kernel log; no flags yet |
| `journalctl` | ⚠️ | `-b`, `-f`, `-k`, `-u`, `-r`, `-n`, `-o`, `-a`, `-c`, `--since`, `--until`, `--follow`, `--rotate`, `-vacuum`, `-S`, `-D`, `--user`, `--disk-usage` | JSON: `-o json`. Rustux: `logs` (C program, step 18; journalctl is another name for it): `-u -t -k -b [N] -p -g -S -U -n -r -f -o short/short-iso/json/cat --list-boots --disk-usage`; no `-c`, `--rotate`, `--vacuum`, `-D`, `--user` |
| `logger` | ⚠️ | `-i`, `-f`, `-p`, `-s`, `-t`, `-u`, `--journald`, `--id` | Rustux: C program (step 18), to logd; `-t -p -s -f -i`; no `-u`, `--journald` |
| `strace` | ⬜ | `-p`, `-f`, `-ff`, `-o`, `-e`, `-s`, `-v`, `-yy`, `-E`, `-c`, `-C` | JSON: `-f -o strace.log` (parseable) |
| `ltrace` | ⬜ | `-c`, `-C`, `-d`, `-e`, `-f`, `-l`, `-n`, `-o`, `-p`, `-s`, `-S`, `-t`, `-x` | |
| `perf` | ⬜ | `record`, `report`, `stat`, `top`, `list`, `script`, `trace`, `kmem`, `lock` | JSON: `report --stdio` (json events) |

---

## 13. Systemd (systemd replacements)

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `systemctl` | ⚠️ | `start`, `stop`, `restart`, `reload`, `status`, `enable`, `disable`, `is-enabled`, `is-active`, `list-units`, `list-unit-files`, `daemon-reload`, `show`, `cat`, `set-property`, `reset-failed`, `list-jobs`, `list-timers`, `list-sockets` | JSON: `--output=json`. Rustux: `svc` (C program, step 18; systemctl is another name for it), units in `/etc/svc`: all but `set-property`, `list-jobs`, `list-timers`, `list-sockets`; also `isolate`, `is-failed`, `get-default`, `poweroff`, `reboot`; `reload UNIT` restarts it; `--json` |
| `bootctl` | ⚠️ | `status`, `list`, `set-default`, `set-oneshot`, `set-timeout`, `reboot-to-firmware`, `install`, `update`, `remove`, `is-installed`, `--esp-path` | JSON: `--json=`. Rustux: C program (step 19) for the boot manager rboot: all but `install`, `update`, `remove` (step 20); no JSON |
| `efibootmgr` | ⚠️ | `-v`, `-o`, `-n`, `-N`, `-b`, `-B`, `-a`, `-A`, `-c`, `-d`, `-p`, `-l`, `-L`, `-t`, `-T`, `-q` | Rustux: C program (step 19): all of these; no `-r`, `-e`, `-@`, `-u` |
| `rustica-install` | ✅ | `--disk`, `--mode`, `--esp`, `--root`, `--user`, `--password-file`, `--lock-root`, `--hostname`, `--timezone`, `--keymap`, `--yes`, `--dry-run` | Rustica's own installer (step 20; like Debian's or Ubuntu's): interactive by default, every change shown before writing |
| `systemd-analyze` | ⬜ | `time`, `blame`, `critical-chain`, `dot`, `dump`, `set-log-level`, `get-log-level`, `verify`, `calendar`, `unit-files`, `unit-paths`, `exit-statuses` | JSON: `--json=` |
| `systemd-cgtop` | ⬜ | `-p`, `-k`, `-K`, `-t`, `-m`, `--batch` | |
| `systemd-resolve` | ⬜ | `status`, `query`, `service`, `openpgp`, `statistics`, `reset-statistics`, `flush-caches` | JSON: `--json=` |
| `systemd-mount` | ⬜ | | |
| `systemd-escape` | ⬜ | `-m`, `-s`, `-p` | |
| `loginctl` | ⬜ | `list-sessions`, `list-users`, `list-seats`, `show-session`, `show-user`, `show-seat`, `activate`, `lock-session`, `unlock-session`, `terminate-session` | JSON: `--output=json` |
| `timedatectl` | ⚠️ | `status`, `set-time`, `set-timezone`, `list-timezones`, `set-ntp`, `set-local-rtc` | JSON: `--json=`. Rustux: C program (step 17); all but `set-ntp` (no network); no JSON |
| `localectl` | ⚠️ | `status`, `set-locale`, `list-locales`, `set-keymap`, `list-keymaps`, `set-x11-keymap`, `list-x11-keymap-models` | JSON: `--json=`. Rustux: C program (step 17); `status`, `set-keymap`, `list-keymaps` (locale is C.UTF-8) |
| `hwclock` | ⬜ | `-r`, `-w`, `-s`, `--show`, `--hctosys`, `--systohc`, `--adjust`, `--getepoch`, `--setepoch`, `--noadjfile`, `--adjfile` | JSON: `--json` |

---

## 14. Archives

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `tar` | ⚠️ | `-c`, `-x`, `-t`, `-v`, `-f`, `-z`, `-j`, `-J`, `-C`, `-p`, `-P`, `--exclude`, `--files-from`, `--transform`, `--verify`, `-W`, `-M` | JSON: `--list --format=json`. Rustux: `tools` (tar + flate2 crates); `c`, `x`, `t`, `z`, `C`, `v`, `O`, `--exclude`; `t --json`; no `-j`/`-J` |
| `gzip` | ⚠️ | `-d`, `-k`, `-l`, `-v`, `-r`, `-S`, `-t`, `-1` to `-9`, `--best`, `--fast` | JSON: `-l`. Rustux: `tools` (flate2); `-d`, `-k`, `-c`, `-f`, `-1`..`-9`; no `-l`, `-r` |
| `gunzip` | ⚠️ | `-c`, `-f`, `-k`, `-l`, `-n`, `-N`, `-r`, `-t`, `-v` | Rustux: `tools` (flate2); `-c`, `-k`, `-f` |
| `zcat` | ✅ | `-f`, `-h`, `-l`, `-L`, `-n`, `-N`, `-q`, `-r`, `-S`, `-v` | Rustux: `tools`, same as `gunzip -c` |
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
| `chmod` | ✅ | `-c`, `-f`, `-v`, `-R`, `--reference`, `--changes`, `--no-preserve-root`, `--preserve-root` | JSON: `--changes`. Rustux: uutils coreutils; modes are stored, not enforced until users exist (step 17) |
| `chown` | ⚠️ | `-c`, `-f`, `-h`, `-R`, `-v`, `--from`, `--reference`, `--changes` | JSON: `--changes`. Rustux: uutils coreutils (step 17); GNU text output, no JSON |
| `chgrp` | ⚠️ | `-c`, `-f`, `-h`, `-R`, `-v`, `--reference`, `--changes` | JSON: `--changes`. Rustux: uutils coreutils (step 17); GNU text output, no JSON |
| `getfacl` | ⬜ | `-a`, `-d`, `-R`, `-p`, `-n`, `--access`, `--default`, `--no-effective`, `--skip-base`, `--all-effective`, `--set`, `--set-file`, `--mask` | JSON: `--no-effective` |
| `setfacl` | ⬜ | `-R`, `-b`, `-k`, `-n`, `-d`, `-m`, `-M`, `-x`, `-X`, `--restore`, `--set`, `--set-file`, `--mask` | |
| `umask` | ⚠️ | `-S`, `-p` | Rustux: shell built-in; `-S` |

---

## 17. Shells

| Command | Status | Key Flags | Notes |
|---------|--------|-----------|-------|
| `bash` | ⬜ | `-c`, `-s`, `-r`, `-i`, `-l`, `-n`, `-x`, `-v`, `--noprofile`, `--norc`, `--rcfile`, `--init-file`, `--dump-strings`, `--dump-po-filenames` | |
| `sh` | ⚠️ | Rustux: link to `shell` (POSIX-like: if/while/for/case, functions, `$(...)`, `$((...))`, pipes, redirection, `#!` scripts) |
| `dash` | ⬜ | | |
| `zsh` | ⬜ | | |
| `env` | ⚠️ | `-i`, `-u`, `--ignore-environment`, `--null`, `-0` | JSON: `-0`. Rustux: uutils coreutils; GNU text output, no JSON yet |
| `printenv` | ⚠️ | `-0`, `--null` | JSON: `-0`. Rustux: uutils coreutils; GNU text output, no JSON yet |
| `set` | ✅ | `-a`, `-b`, `-e`, `-u`, `-x`, `-o`, `-h` | Rustux: shell built-in |
| `unset` | ✅ | `-f`, `-v` | Rustux: shell built-in |
| `export` | ✅ | `-n`, `-p`, `-f` | Rustux: shell built-in |
| `source` | ✅ | Rustux: shell built-in |
| `alias` | ✅ | `-p` | Rustux: shell built-in |
| `unalias` | ✅ | `-a` | Rustux: shell built-in |
| `history` | ⚠️ | `-c`, `-d`, `-a`, `-n`, `-r`, `-w`, `-p`, `-s` | JSON: `-w -n`. Rustux: shell built-in; lists only |
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

Statuses are for Rustux (checklist step 11 of the rustux repo). Its `docs/COMMANDS.md` lists where each command comes from and how it differs from GNU. Most commands are uutils coreutils and keep GNU's text output, so a row that asks for JSON stays ⚠️ until JSON is added.

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

1. **Phase 1 - Core Package Management**: `rpg` (`PACKAGES.md`, Rustux phase E)
2. **Phase 2 - Basic System Info**: `uname`, `lsb_release`, `hostname`, `free`, `df`, `lsblk`
3. **Phase 3 - File Operations**: `ls`, `cp`, `mv`, `rm`, `mkdir`, `cat`
4. **Phase 4 - Network**: `ip`, `ss`, `ping`, `curl`, `ssh`
5. **Phase 5 - Process**: `ps`, `top`, `kill`, `killall`
6. **Phase 6 - User Management**: `useradd`, `userdel`, `usermod`, `passwd`
7. **Phase 7 - System Control**: `systemctl` replacement, `reboot`, `shutdown`
