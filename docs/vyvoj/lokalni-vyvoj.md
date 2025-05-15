---
title: Spuštění vývojové verze
---

## Požadavky

Před spuštěním vývojové verze je nutné mít na počítači nainstalováno následující:

- **npm** (Node.js balíčkovací systém)
- **Rust** (programovací jazyk)
- **Tectonic** (LaTeX distribuce)
- **AIDE** (Advanced Intrusion Detection Environment)

## Postup spuštění

1. **Stažení projektu z GitHubu**  
   Nejprve stáhněte projekt z GitHubu pomocí příkazu:

   ```bash
   git clone https://github.com/cyber-rangers/CRATEC.git
   ```

2. **Přechod do složky projektu**  
   Přesuňte se do složky projektu:

   ```bash
   cd CRATEC
   ```

3. **Instalace závislostí**  
   Nainstalujte všechny potřebné závislosti pomocí příkazu:

   ```bash
   npm i
   ```

4. **Stažení a konfigurace AIDE**  
   - Stáhněte konfigurační soubor `aide.conf` z [releases na GitHubu](https://github.com/cyber-rangers/CRATEC/releases).
   - Přesuňte soubor do `/etc/aide.conf`:

     ```bash
     sudo mv aide.conf /etc/aide.conf
     ```

   - Inicializujte AIDE:

     ```bash
     sudo aide --init
     ```

   - Přesuňte databázový soubor:

     ```bash
     sudo mv /var/lib/aide/aide.db.new.gz /var/lib/aide/aide.db.gz
     ```

5. **Spuštění vývojové verze**  
   Spusťte vývojovou verzi aplikace pomocí příkazu:

   ```bash
   npm run tauri dev
   ```

6. **Build aplikace**  
   Pokud chcete vytvořit build aplikace, použijte příkaz:

   ```bash
   npm run tauri build
   ```

## Troubleshooting

### Problémy s Tectonic
Pokud Tectonic nefunguje správně, zkuste nastavit následující proměnnou prostředí:

```bash
export CXXFLAGS="-std=c++17"
```

### Problémy s buildem aplikace
Pokud se aplikace nechce buildnout, zkuste použít následující příkaz:

```bash
NO_STRIP=TRUE npm run tauri build
```

Tímto by měly být vyřešeny nejčastější problémy při spuštění vývojové verze.