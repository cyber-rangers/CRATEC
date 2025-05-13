---
title: Aplikace
---

# Instalace aplikace

## Dokončení instalace systému pomocí instalačního skriptu

### Krok 1: Příprava prostředí
1. Při restartu zařízení předchozí instalaci Arch Linuxu vstupte do firmwaru (klávesa **ESC**)
2. V sekci `Security → Secure Boot`:
   - Zvolte režim **Custom**
   - Odstraňte všechny existující klíče
   - Přepněte do **Setup Mode**
3. Uložte změny a spusťte systém

:::important
Secure Boot zatím **neaktivujte**! Tento krok provedeme až na konci celého procesu.
:::

### Krok 2: První spuštění systému
- Při bootování zadejte heslo k LUKS oddílu: `cratec`
- Přihlaste se pomocí:
  ```bash
  Uživatel: cratec
  Heslo: cratec
  ```

### Krok 3: Stažení instalačního skriptu
```bash
curl -O https://github.com/[ODKAZ-DOPLNIT]/install.sh
chmod +x install.sh
```

### Krok 4: Spuštění instalačního skriptu
```bash
sudo ./install.sh [parametr]
```

#### Dostupné parametry:
| Parametr            | Popis                                        |
| ------------------- | -------------------------------------------- |
| `diskencryption`    | Nastaví automatické odemykání pomocí TPM 2.0 |
| `secureboot`        | Připraví prostředí pro Secure Boot           |
| `application`       | Nainstaluje aplikaci CRATEC                  |
| `full`              | Spustí všechny části postupně                |
| `--luks-pass HESLO` | Umožní zadat alternativní LUKS heslo         |

Pro nápovědu:
```bash
./install.sh --help
```

#### Části instalačního skriptu

1. **Disk Encryption (TPM 2.0)**
   - Nastaví automatické odemykání LUKS přes TPM čip
   - Aktualizuje konfiguraci initramfs
   - Odstraní původní heslo pro root oddíl

2. **Secure Boot Preparation**
   - Vyčistí existující boot záznamy
   - Vygeneruje a nahraje vlastní Secure Boot klíče
   - Podepíše systémové binární soubory
:::warning[Omezení virtuálních prostředí]
Nastavení Secure Boot **není možné replikovat na virtuálních strojích** (VirtualBox, VMware, Hyper-V) z důvodu chybějící podpory pro **Setup Mode**. Tato konfigurace vyžaduje přímý přístup k UEFI firmware fyzického zařízení.
:::

3. **Aplikace CRATEC**
   - Vytvoří uživatele `cratec` s potřebnými právy
   - Nainstaluje všechny závislosti a aplikaci
   - Nastaví automatické přihlášení a kiosk režim
   - Inicializuje kontrolu integrity souborů (AIDE)

:::note
Po použití `diskencryption` bude zařízení závislé výhradně na TPM čipu pro odemykání disku.
:::

### Krok 5: Finalizace instalace
1. Restartujte zařízení:
   ```bash
   sudo reboot
   ```
2. Vstupte znovu do firmwaru (klávesa **ESC**)
3. V `Secure Boot` aktivujte režim **Enabled**

:::danger POSLEDNÍ KROK
Po aktivaci Secure Boot již systém **nebude bootovat** s nepodepsanými binárními soubory! Ujistěte se, že všechny části skriptu byly úspěšně dokončeny.
:::


 Po restartu se aplikace automaticky spustí