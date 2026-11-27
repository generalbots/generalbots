# Código-fonte dos componentes copyleft / Source of the copyleft components

General Bots Lite para Windows. Este arquivo vai no instalador, na pasta
`licenses`.

## Português

Alguns programas que o instalador traz, ou que o aplicativo baixa quando um
serviço é ligado, têm licenças que exigem que o código-fonte esteja
disponível para quem recebe o programa (AGPL, MPL e LGPL). As tabelas dizem,
para cada um, onde está o código-fonte da versão exata.

Nenhum desses programas foi alterado pelo General Bots. A MinIO foi
compilada a partir do código original, sem mudanças, porque o autor não
publicou o executável da versão que corrige a falha de segurança
CVE-2025-62506. A receita dessa compilação também está indicada abaixo.

Se algum endereço deixar de funcionar, peça uma cópia do código-fonte
abrindo um chamado em https://github.com/generalbots/generalbots/issues.

O código do próprio General Bots é MIT, e está todo em
https://github.com/generalbots/generalbots.

## English

Some programs that this installer carries, or that the app downloads when a
service is switched on, have licences that require their source code to stay
available to whoever receives the program (AGPL, MPL and LGPL). The tables
say, for each one, where the source of the exact version is.

General Bots changed none of them. MinIO was compiled from the original
source, unchanged, because its author published no executable for the
release that fixes CVE-2025-62506. The recipe for that build is listed
below as well.

If an address stops working, ask for a copy of the source by opening an
issue at https://github.com/generalbots/generalbots/issues.

General Bots' own code is MIT, all of it at
https://github.com/generalbots/generalbots.

## Inside the installer / Dentro do instalador

| Component | Version | Licence | Source of that exact version |
|---|---|---|---|
| MinIO server (`minio.exe`) | RELEASE.2025-10-15T17-29-55Z, commit 9e49d5e7a648f00e26f2246f4dc28e6b07f8c84a | AGPL-3.0-or-later | Source archive: https://github.com/minio/minio/archive/refs/tags/RELEASE.2025-10-15T17-29-55Z.tar.gz (SHA-256 be6d0bd3696c3a13a35f02d3a0280b64319c67918b4501c5c3d87f96d000085c). Build recipe (Go 1.24.8, `CGO_ENABLED=0 GOOS=windows GOARCH=amd64 go build -trimpath` with version flags only): https://github.com/generalbots/generalbots/blob/ad6f6b665646eadc3f86cb0e38a11cc879080feb/.github/workflows/drive-bin-v4.yml. The resulting `minio.exe` (SHA-256 e63f9c4ac8b8697ca679d4864497688c3486aa9ac9370c3569203420c25e5799) is published at https://github.com/generalbots/generalbots/releases/tag/drive-bin-v4 |
| MinIO Client (`mc.exe`) | RELEASE.2025-08-13T08-35-41Z | AGPL-3.0-or-later | https://github.com/minio/mc/tree/RELEASE.2025-08-13T08-35-41Z (upstream binary, unchanged) |
| OpenBao (`bao.exe`) | v2.7.3 | MPL-2.0 | https://github.com/openbao/openbao/tree/v2.7.3 (upstream binary, unchanged) |
| GNU libiconv (`libiconv-2.dll` in `installers`) | 1.15, read from the file's version stamp | LGPL-2.0-or-later (text: `LGPL-2.0.txt`) | The DLL comes from the PostgreSQL 17.4-1 Windows binaries published by EDB. Upstream source of version 1.15: https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.15.tar.gz (5,264,188 bytes), signature https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.15.tar.gz.sig . SHA-256: not hashed (GNU publishes the signature only; the tarball was not downloaded). The source EDB built from is not located |
| GNU libiconv (`libiconv-2.dll` inside the PostgreSQL 18.6-1 zip, `pgsql/bin`) | 1.19, read from the file's version stamp | LGPL-2.1-or-later (text: `LGPL-2.1.txt`) | The zip is the PostgreSQL 18.6-1 Windows binaries published by EDB, carried whole. Upstream source of version 1.19: https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.19.tar.gz (5,921,103 bytes), signature https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.19.tar.gz.sig . SHA-256: not hashed. The source EDB built from is not located |
| GNU gettext libintl (`libintl-9.dll` in `installers`, and the same file inside the PostgreSQL 18.6-1 zip) | 0.19.8, read from the file's version stamp | LGPL-2.1-or-later (text: `LGPL-2.1.txt`) | From the same EDB binaries. Upstream source of version 0.19.8: https://ftp.gnu.org/pub/gnu/gettext/gettext-0.19.8.tar.gz (19,977,131 bytes), signature https://ftp.gnu.org/pub/gnu/gettext/gettext-0.19.8.tar.gz.sig . GNU's release announcement (https://lists.gnu.org/archive/html/info-gnu/2016-06/msg00003.html) gives MD5 7ad5c90e32ac6828de955a0432ab4a7c and SHA-1 4b3dc947a729fac8a14bde473454730919382b23 for that file. SHA-256: not hashed (GNU published none; the tarball was not downloaded). The source EDB built from is not located |

## Not in the installer: downloaded only if the service is switched on / Fora do instalador: baixado só se o serviço for ligado

| Component | Version | Licence | Source of that exact version |
|---|---|---|---|
| Zitadel (`zitadel.exe`) | v4.20.1 | AGPL-3.0-only; parts Apache-2.0 and MIT (see `zitadel-LICENSING.md`) | https://github.com/zitadel/zitadel/tree/v4.20.1 (upstream binary, unchanged, fetched from https://github.com/zitadel/zitadel/releases/download/v4.20.1/zitadel-windows-amd64.tar.gz) |

Zitadel stays in this file although the installer no longer carries it
(since 2026-11-27). The directory service is switched off in this edition,
but the app still downloads exactly this version when the service is
switched on (`GB_SERVICES_ON=directory`), and installers before that date
carried it, so a copy can be on a computer that was updated.

O Zitadel continua neste arquivo, embora o instalador não o traga mais
(desde 2026-11-27). O serviço de diretório vem desligado nesta edição, mas o
aplicativo ainda baixa exatamente esta versão quando o serviço é ligado
(`GB_SERVICES_ON=directory`), e os instaladores anteriores a essa data o
traziam; por isso pode haver uma cópia num computador que foi atualizado.

The MinIO, mc, Zitadel and OpenBao repositories are Go programs. Their
tagged source pins every dependency in `go.mod` and `go.sum`; `go mod
download` at that tag fetches them.

How the versions and licences of the two GNU libraries were established
(version stamps, the sentences of the GNU release notes and licence files,
with their addresses) is in `LICENSES-desktop.md`, section 1.2.

## Not settled yet / Ainda não resolvido

This file records where the source is. Whether this meets each licence is
a legal question that has not been answered yet:

- AGPL-3.0 §6 and MPL-2.0 §3.2: whether links to upstream tags and to the
  build recipe are enough, or whether the distributor must host a copy.
- Whether "Corresponding Source" must include the Go module dependencies
  as files.
- LGPL DLLs: the versions are known now. The exact source and build recipe
  EDB used are not: EDB's own licence list inside the 18.6-1 zip
  (`pgsql/commandlinetools_3rd_party_licenses.txt`) gives the LGPL 2.1 text
  for "gettext, libiconv, pthreads" and no source address or written offer.
- libiconv 1.15: two files of its Windows DLL build carry a GPL header in
  that release, which its maintainer later called an oversight and changed
  (https://lists.gnu.org/archive/html/bug-gnu-libiconv/2019-05/msg00003.html).
  Whether EDB's build includes them is not known.
- The PostgreSQL 18.6-1 zip is carried whole, with pgAdmin 4 and
  StackBuilder inside. EDB's list for pgAdmin
  (`pgsql/pgAdmin_3rd_party_licenses.txt`) names further LGPL parts ("QT,
  gettext, libiconv, pthreads"). Their versions and source were not checked.
- Whether a download that the app starts from an upstream release page
  (Zitadel) counts as distribution by General Bots.
