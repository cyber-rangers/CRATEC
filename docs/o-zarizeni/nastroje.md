---
title: Nástroje
---

# Nástroje

Aplikace využívá následující nástroje, které jsou napsány v jazyce Rust nebo jsou unixové utility. Tyto nástroje umožňují přímou práci s fyzickým hardwarem bez závislosti na komplexních API. Výběr byl motivován jejich stabilitou, transparentností a širokou podporou v prostředí GNU/Linux.

## Seznam nástrojů

1. **[ewfacquire](https://github.com/libyal/libewf)**  
   Součást knihovny libewf, která umožňuje vytváření a manipulaci s EWF (Expert Witness Format) obrazy disků. Tento nástroj je klíčový pro forenzní akvizici dat.

2. **[dcfldd](https://github.com/resurrecting-open-source-projects/dcfldd)**  
   Rozšířená verze nástroje `dd`, která podporuje výpočet hashů během kopírování dat. Používá se při vytváření bitových kopií disků.

3. **[hdparm](https://sourceforge.net/projects/hdparm/)**  
   Nástroj pro získání podrobných informací o discích na nízké úrovni. Umožňuje detekovat skryté oblasti HPA (Host Protected Area) a DCO (Device Configuration Overlay), kontrolovat ATA parametry a provádět pokročilé příkazy pro diagnostiku a ověření stavu zařízení.

4. **[lsblk](https://github.com/util-linux/util-linux)**  
   Nástroj pro zobrazení blokových zařízení a jejich struktury. Slouží k přesné identifikaci jednotlivých diskových oddílů, jejich velikosti a typu.

5. **[parted](https://gitlab.com/parted/parted)**  
   Nástroj pro správu diskových oddílů. Používá se k analýze a konfiguraci diskových struktur.

6. **[sha1sum, md5sum, sha256sum](https://www.gnu.org/software/coreutils/)**  
   Jednoduché unixové nástroje pro výpočet kryptografických hashů. Používají se k ověření integrity dat při akvizici i při kontrole výsledných obrazů.

7. **[AIDE (Advanced Intrusion Detection Environment)](https://github.com/aide/aide)**  
   Nástroj pro kontrolu integrity systému. Na základě databáze otisků souborů dokáže odhalit změny ve spustitelných binárkách, konfiguracích i systémových knihovnách.

8. **[Tectonic](https://github.com/tectonic-typesetting/tectonic)**  
   Moderní překladač LaTeX dokumentů, který je integrován pro automatizované generování výstupních zpráv v PDF formátu. Na rozdíl od klasického TeX toolchainu nevyžaduje externí distribuci (např. TeX Live).

