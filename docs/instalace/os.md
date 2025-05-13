---
title: Arch Linux
---

## 1. Příprava instalačního média

### 1.1 Stažení ISO obrazu
1. Navštivte [oficiální stránky Arch Linuxu](https://archlinux.org/download/)
2. Stáhněte nejnovější stabilní verzi ISO souboru
3. Ověřte kontrolní součet (SHA256)

### 1.2 Vytvoření bootovacího USB
1. Spusťte aplikaci Rufus (Windows) nebo `dd` (Linux)
2. Nastavení v Rufusu:
   ```plaintext
   Zařízení: Váš USB disk (min. 2GB)
   Výběr bootovacího souboru: Stažené ISO
   Schéma oddílu: GPT
   Cílový systém: UEFI (non-CSM)
   Systém souborů: FAT32
   Velikost clusteru: 4096 bajtů
   ```
3. Klikněte na "Start" a počkejte na dokončení

## 2. Spuštění instalace

### 2.1 Bootování z USB
1. Vložte USB disk do cílového zařízení
2. Při startu počítače stiskněte boot menu klávesu (obvykle F12, ESC nebo DEL)
3. Vyberte své USB zařízení z boot menu

### 2.2 Spuštění instalátoru
Po načtení instalačního prostředí:
```bash
archinstall
```

## 3. Podrobná konfigurace

### 3.1 Lokalizace
```bash
Keyboard layout: cz (Czech qwerty)
Locales:
  - cs_CZ.UTF-8 (hlavní locale)
  - en_US.UTF-8 (záložní locale)
Timezone: Europe/Prague
```

### 3.2 Diskové nastavení
```bash
Disk: /dev/nvme0n1 (NVMe SSD)
Rozdělení disku:
  - Automatické (defaultní schéma)
  - Celý disk jako jeden oddíl
Souborový systém: ext4
Šifrování: LUKS (AES-XTS 512bit)
  - Heslo: cratec (pro testovací účely)
Swap: Povolen (1/2 velikosti RAM)
```

### 3.3 Systémové nastavení
```bash
Zavaděč: systemd-boot (pro UEFI)
Unified Kernel Images: Zakázáno
Hostname: CRATEC
Root účet: Pouze sudo, žádné heslo
Hlavní uživatel:
  - Jméno: cratec
  - Heslo: cratec
  - Oprávnění: sudo (superuživatel)
```

### 3.4 Balíčky a repozitáře
```bash
Profil instalace: Minimal (základní systém)
Mirror region: Czechia (automatický výběr zrcadla)
Dodatečné balíčky: Žádné
Síť: NetworkManager (pro správu připojení)
```

### 3.5 Časové nastavení
```bash
Časová zóna: Europe/Prague
NTP: Povoleno (automatická synchronizace času)
```

## 4. Dokončení instalace

1. Zkontrolujte nastavení
2. Spusťte instalaci:
   ```bash
   Install
   Potvrdit instalaci? [Y/n] Y
   ```
3. Po dokončení zvolte že nechcete do post-instalačního terminálu a restartujte:
   ```bash
   sudo reboot
   ```
4. Postupujte dále návodem pro instalaci aplikace


:::danger DŮLEŽITÉ BEZPEČNOSTNÍ UPOZORNĚNÍ
Je důležité mít totožnou instalaci OS, co byla zde popsána, jinak instalační skript v následujícím kroku nebude fungovat.
:::
