
# CRATEC

Zařízení pro vytváření bitových kopií disků pro účely forenzní analýzy.




## Tech Stack

**Frontend:** [Tauri](https://tauri.app/), [SvelteKit](https://kit.svelte.dev/), [Skeleton](https://www.skeleton.dev/)

**Backend:** [Rust](https://www.rust-lang.org/)


## Instalace

Instalace knihoven

```bash
  npm i 
```
```bash
  cargo build
```

Spuštění Vývojové verze
```bash
  cargo tauri dev
```

## Hardware

| Produkt                                                                                            | Obchod    |Účel | Cena za 1 ks                |
|----------------------------------------------------------------------------------------------------|-----------|-----------|-----------------------------|
| [ODROID-H4 ULTRA](https://rpishop.cz/516778/odroid-h4-ultra/)                                       | RPishop   | Jednodeskový počítač | 7 539 Kč                    |
| [19V/7A Power Supply](https://rpishop.cz/zdroje-s-21x55mm-kabelem/5545-21x55mm-19v7a-napajeci-zdroj-eu-cerny.html) | RPishop  | Napájení počítače | 869 Kč                      |
| [SATA Data and Power Cable 200mm (na testování)](https://rpishop.cz/sata/1686-hardkernel-sata-datovy-a-napajeci-kabel.html) | RPishop   |Testovací kabel na propojení SATA| 109 Kč                      |
| [Waveshare 7" IPS LCD integrovatelný displej, 1024×600](https://rpishop.cz/lcd-oled-displeje/5778-waveshare-7-ips-integrovatelny-displej-1024600-hdmi-ips-lcd-dotykovy-kapacitni.html) | RPishop   | Dotykový display | 1 175 Kč                    |
| [Crucial SO-DIMM 32GB DDR5 4800MHz CL40](https://www.alza.cz/crucial-so-dimm-32gb-ddr5-4800mhz-cl40-d7987959.htm?o=1) | Alza      |RAM paměť | 2 379 Kč                    |
| [WD Red SN700 NVMe 250GB](https://www.alza.cz/wd-red-sn700-nvme-250gb-d6998053.htm)                 | Alza      | SSD pro OS | 1 339 Kč                    |
| [2x Noctua NF-A4x20 PWM](https://www.alza.cz/noctua-nf-a4x20-pwm-d5075791.htm?o=7#description)         | Alza      |PWM větráky | 718 Kč                      |
| [Napájecí kabel 230V (1m)](https://www.alza.cz/kabel-napajeci-230v-k-d88978.htm)                    | Alza      | Kabel ke zdroji | 119 Kč                      |
| [OEM Prodlužovací SATA datový a napájecí, 0.5m](https://www.alza.cz/prodluzovaci-sata-datovy-a-napajeci-50-cm-d360854.htm) | Alza      | Propojovací kabel disku a obalu zařízení| 119 Kč                      |
| [ADT-Link USB3.1 Gen2 (verze S2A – T4A 7/15cm)](https://www.aliexpress.com/item/1005003510149476.html) | Aliexpress | Propojovací kabel PC a dotyku displeje| 384 Kč                      |
| [ADT-Link HDMI to HDMI Type (verze A2 – A1R 15cm)](https://www.aliexpress.com/item/1005003489166473.html) | Aliexpress |Propojovací kabel PC a obrazu displeje| 290 Kč                      |
| [SATA 7+15Pin Floppy Drive To Small 4Pin 2.54MM 0.3/0.2m](https://www.aliexpress.com/item/1005005717342937.html) | Aliexpress | Propokovací kabel SATA portů PC a obalu zařízení| 193 Kč                      |
||||Celkem|15 525Kč |

### ODROID-H4 ULTRA
![odroid](https://rpishop.cz/wp-content/uploads/2024/05/ORD211_3.jpg)

![odroid](https://rpishop.cz/wp-content/uploads/2024/05/ORD211_2.jpg)

#### Paměť
- **Slot:** 1× DDR5 SO-DIMM (jednokanálová paměť)
- **Rychlost:** 4800 MT/s
- **Maximální kapacita:** 48 GB

#### Úložiště
- **eMMC:** 1× konektor (bootovatelný a volitelný v systému BIOS)
- **SATA3:** 4× sloty, 6 Gb/s
- **NVMe M.2:** 1× slot (PCIe Gen 3 ×4)

#### Síťové Připojení
- **LAN Porty:** 2× 2.5 GbE LAN port (RJ45, podpora 10/100/1000/2500 Mb/s)

#### Video
- **DisplayPort:** 2× DisplayPort 1.2 (až 4K@60Hz)
- **HDMI:** 1× HDMI 2.0 (až 4K@60Hz)
- **Podpora:** Trojí současné zobrazení

#### Zvuk
- **Audio Výstup:** 1× 3,5 mm jack
- **Audio Vstup:** 1× 3,5 mm jack
- **SPDIF Výstup:** 1× (ALC662, kodek HDA)
- **Další:** HDMI a DisplayPort mají také zvukový výstup

#### Externí I/O
- **USB Porty:**
  - 2× USB 3.0 hostitelské porty
  - 2× USB 2.0 hostitelské porty
- **Rozšiřující Slot pro Periferní Zařízení:**
  - 24 pinů, rozteč 2,54 mm
  - 1× DC 5V
  - 1× DC 3,3V
  - 5× GND
  - 1× UART (TXD/RXD/RTS/CTS)
  - 2× I2C (SCL/SDA)
  - 1× Tlačítko externího napájení

#### Další Funkce
- **Chlazení:**
  - Pasivní chladič
  - Konektor pro aktivní chladicí ventilátor (volitelný)
    - **Specifikace:** 12V 4pin, vstup PWM + výstup TACHO
- **Záložní Baterie Systému BIOS:**
  - Udržuje systémový čas a nastavení systému BIOS
- **Tlačítka:**
  - Tlačítko napájení
  - Tlačítko resetování
#### Napájení
- **DC Konektor:** 
  - Vnější (záporný) průměr 5,5 mm
  - Vnitřní (kladný) průměr 2,1 mm


### Waveshare 7" IPS LCD integrovatelný displej

![wave](https://rpishop.cz/wp-content/uploads/2023/02/27431-Waveshare-7-IPS-LCD-integrovatelny-displej-1024600-HDMI-dotykovy-kapacitni.jpg)
![wave](https://www.waveshare.com/img/devkit/LCD/70H-1024600/70H-1024600-details-intro.jpg)

#### Vlastnosti

- **Úhlopříčka:** 7″
- **Rozlišení:** 1024×600
- **Zobrazovací panel:** IPS
- **Pozorovací úhel:** 170°
- **Typ dotyku:** kapacitní
- **Dotykové body:** 5 bodů
- **Spotřeba:** nízká
