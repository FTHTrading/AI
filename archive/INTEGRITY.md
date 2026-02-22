# Genesis Protocol v1.0.0 — Archive Integrity Record

## Archive File

| Field | Value |
|---|---|
| Filename | Genesis-Protocol-v1.0.0-Experimental-Engine.tar.gz |
| SHA-256 | 450FFF3170B919CFD6374B4B205A20ADC694FBDDF624517D3F97896EA245D7DD |
| Size | 1.33 MB |
| Created | 2026-02-22 |
| Git Commit | 1955dfa900296065308be5dcd232c580e9e8ef9a |
| Method | `git archive --format=tar.gz --prefix="Genesis-Protocol-v1.0.0/" HEAD` |

## IPFS Record

| Field | Value |
|---|---|
| CID | _PENDING — Upload required_ |
| Gateway URL | _PENDING_ |
| Pin Service | _PENDING_ |

## Verification

To verify the archive:

```powershell
# On Windows
(Get-FileHash "Genesis-Protocol-v1.0.0-Experimental-Engine.tar.gz" -Algorithm SHA256).Hash
# Expected: 450FFF3170B919CFD6374B4B205A20ADC694FBDDF624517D3F97896EA245D7DD
```

```bash
# On Linux/macOS
sha256sum Genesis-Protocol-v1.0.0-Experimental-Engine.tar.gz
# Expected: 450fff3170b919cfd6374b4b205a20adc694fbddf624517d3f97896ea245d7dd
```

To verify the archive matches the commit:

```bash
git clone https://github.com/FTHTrading/AI.git
cd AI
git checkout 1955dfa900296065308be5dcd232c580e9e8ef9a
git archive --format=tar.gz --prefix="Genesis-Protocol-v1.0.0/" HEAD > rebuild.tar.gz
sha256sum rebuild.tar.gz
# Must match the hash above
```

---

*This record serves as timestamped proof of existence for the Genesis Protocol system at the specified commit.*
