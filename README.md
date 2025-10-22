# CTF Challenges Repository

## Start CTFd

```bashbash
cd ctfd && docker-compose up
```

## Start challenges

```bash
cd challenges && docker-compose up
```

## CTFd is running at

http://localhost:7000

## Sync the challenges

Modify the token at ./.ctfcli/config before running this command

```bashbash
python3 -m pip install ctfcli
python3 -m ctfcli challenges install
python3 -m ctfcli challenges sync
```

## Install challenges (not needed)

```bashbash
python3 -m pip install ctfcli
python3 -m ctfcli init http://localhost:7000
for dir in challenges/*/; do python3 -m ctfcli challenge add $dir; done
```

# All credits to https://github.com/BSidesSF/ctf-2025-release and https://github.com/CTFd/CTFd