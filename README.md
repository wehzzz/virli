# virli

## Authors

- anton.vella
- martin.levesque

## How to run

```sh
make
./out/mymoulette -I library/node:19-alpine /bin/sh
```

## Implementation

### Palier 0 : Bonnes pratiques, propreté et architecture
- **Files:** `main.rs`, `parse.rs`.

### Palier 1 : Restreindre l’environnement
- **Files:** `cgroup.rs`.
- **Difficulties:** Writing to `/sys/fs/cgroup/` as a standard user is forbidden. Resolved by granting the `CAP_DAC_OVERRIDE` capability to the binary to bypass file permissions during configuration.

### Palier 2 : Réduire les capabilities
- **Files:** `capabilities.rs`.

### Palier bonus : Utilisable par un utilisateur
- **Files:** `Makefile`.
- **Difficulties:** We had added several capabilities which we then removed.

### Palier 3 : Isolation du pauvre
- **Files:** `chroot.rs`. (unused, replaced by pivot_root)

### Palier 4 : seccomp
- **Files:** `seccomp.rs`.

### Palier 5 : Automatisation de la création de l’environnement
- **Files:** `oci.rs`.
- **Difficulties:** Understanding how to retrieve layers via the manifest. Occasionally, a manifest list is returned in which we must search for the manifest related to the desired architecture (amd64/linux).

### Palier 6 : Une vraie isolation
- **Files:** `namespace.rs`.

### Palier bonus : Révolution des utilisateurs

Container execution without being root on the host machine.
- **Files:** `namespace.rs`.
- **Difficulties:** The configuration of maps (`uid_map`, `gid_map`) is very sensitive to the order of operations. Writing `setgroups=deny` is mandatory before GID mapping to avoid an `EPERM` error. We encountered problems because we had to leave a capability in the `Makefile`, otherwise we could not create the cgroups. The problem is that when setting capabilities, the DUMPABLE flag is set to `0`, and if it is not reset to `1` later, we cannot write to `uid_map`, `gid_map`.

### Palier 7 : Empêcher les fuites d’information
- **Files:** `mounts.rs`.

### Palier 7 bis : Volume attaché au code à moulinéter
- **Files:** `mounts.rs`.

### Palier 8 : Identification du conteneur
- **Files:** `namespace.rs`.

### Palier 9 : pivot_root
Root replacement via `pivot_root` (more secure than `chroot`).
- **Files:** `chroot.rs`.

### Palier 10 : Bac à sable connecté
We did not implement this level.

### Bonus : OverlayFS
We mounted the rootfs in overlayfs to prevent changes to the image from persisting between runs. We were trying to approximate the behavior of docker.
- **Files:** `mount.rs`.